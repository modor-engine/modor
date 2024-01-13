use crate::components::instance_group::{InstanceDataUpdateQuery, InstanceGroup2DRegistry};
use crate::components::renderer::GpuContext;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::TextureRegistry;
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{
    errors, InstanceData, InstanceGroup2D, InstanceRendering2D, MaterialSource, Renderer, Texture,
    DEFAULT_SHADER,
};
use derivative::Derivative;
use modor::{Component, ComponentSystems, Custom, Query, SingleRef};
use modor_resources::{
    ResKey, Resource, ResourceAccessor, ResourceAccessorMut, ResourceLoadingError,
    ResourceRegistry, ResourceState,
};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::{any, mem};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource};

pub(crate) type MaterialRegistry = ResourceRegistry<Material>;

/// A material that defines the aspect of a rendered instance.
///
/// # Requirements
///
/// The material is effective only if:
/// - graphics [`module`](crate::module()) is initialized
/// - the entity contains components of type [`MaterialSync<M>`] and `M`
///
/// # Related components
///
/// - [`MaterialSync`]
/// - [`InstanceRendering2D`](InstanceRendering2D)
///
/// # Entity functions creating this component
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
/// - [`material`](crate::material())
///
/// # Examples
///
/// See [`instance_group_2d`](crate::instance_group_2d()),
/// [`instance_2d`](crate::instance_2d()) and [`material`](crate::material())
/// as most of the time these methods will be used to create a material.
///
/// See [`MaterialSource`] for custom material data.
#[must_use]
#[derive(Component, Debug)]
pub struct Material {
    pub(crate) shader_key: ResKey<Shader>,
    pub(crate) is_transparent: bool,
    pub(crate) bind_group: Option<BindGroup>,
    pub(crate) texture_keys: Vec<ResKey<Texture>>,
    pub(crate) instance_data_size: usize,
    pub(crate) instance_data_type: TypeId,
    key: ResKey<Self>,
    buffer: Option<DynamicBuffer<u8>>,
    renderer_version: Option<u8>,
    error: Option<ResourceLoadingError>,
}

#[systems]
impl Material {
    /// Creates a new material with a unique `key`.
    pub fn new(key: ResKey<Self>) -> Self {
        Self {
            shader_key: DEFAULT_SHADER,
            is_transparent: false,
            bind_group: None,
            texture_keys: vec![],
            instance_data_size: 0,
            instance_data_type: TypeId::of::<()>(),
            key,
            buffer: None,
            renderer_version: None,
            error: None,
        }
    }

    fn update<M>(
        &mut self,
        renderer: &Option<SingleRef<'_, '_, Renderer>>,
        shaders: Custom<ResourceAccessor<'_, Shader>>,
        textures: Custom<ResourceAccessor<'_, Texture>>,
        source: &M,
    ) where
        M: MaterialSource,
    {
        self.instance_data_size = mem::size_of::<M::InstanceData>();
        self.instance_data_type = TypeId::of::<M::InstanceData>();
        let state = Renderer::option_state(renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.bind_group = None;
            self.buffer = None;
        }
        if let Some(context) = state.context() {
            let shader_key = source.shader_key();
            let texture_keys = source.texture_keys();
            if let (Some(shader), Some(textures)) = (
                shaders.get(shader_key),
                texture_keys
                    .iter()
                    .map(|&k| textures.get(k))
                    .collect::<Option<Vec<&Texture>>>(),
            ) {
                self.update_buffer(context, source);
                if self.bind_group.is_none()
                    || self.texture_keys != texture_keys
                    || self.shader_key != shader_key
                    || shader.is_material_bind_group_layout_reloaded
                    || textures.iter().any(|t| t.is_reloaded)
                {
                    self.update_bind_group(shader, &textures, context);
                }
                self.is_transparent = source.is_transparent()
                    || (!shader.is_alpha_replaced()
                        && textures.iter().any(|t| t.inner().is_transparent));
                self.texture_keys = texture_keys;
                self.shader_key = shader_key;
                if shader.material_instance_type != self.instance_data_type {
                    self.error = Some(ResourceLoadingError::LoadingError(format!(
                        "mismatch instance data type (`{}` in material, `{}` in shader)",
                        any::type_name::<M::InstanceData>(),
                        shader.material_instance_type_name
                    )));
                }
            } else {
                self.bind_group = None;
                self.buffer = None;
            }
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_bind_group(&mut self, shader: &Shader, textures: &[&Texture], context: &GpuContext) {
        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: self
                .buffer
                .as_ref()
                .expect("internal error: material buffer not initialized")
                .resource(),
        }];
        for (i, texture) in textures.iter().enumerate() {
            entries.extend([
                BindGroupEntry {
                    binding: (i * 2 + 1) as u32,
                    resource: BindingResource::TextureView(&texture.inner().view),
                },
                BindGroupEntry {
                    binding: (i * 2 + 2) as u32,
                    resource: BindingResource::Sampler(&texture.inner().sampler),
                },
            ]);
        }
        let binding_group = errors::validate_wgpu(context, || {
            context.device.create_bind_group(&BindGroupDescriptor {
                layout: shader
                    .material_bind_group_layout
                    .as_ref()
                    .expect("internal error: material bind group not initialized"),
                entries: &entries,
                label: Some(&format!("modor_bind_group_material_{}", self.key.label())),
            })
        });
        match binding_group {
            Ok(binding_group) => {
                self.bind_group = Some(binding_group);
                self.error = None;
            }
            Err(error) => {
                self.bind_group = None;
                self.error = Some(ResourceLoadingError::LoadingError(format!(
                    "maybe the number of textures in the material does not match the shader code: \
                     {error}"
                )));
            }
        }
    }

    fn update_buffer(&mut self, context: &GpuContext, source: &impl MaterialSource) {
        let data = Vec::from(bytemuck::try_cast_slice(&[source.data()]).unwrap_or(&[0]));
        if let Some(buffer) = &mut self.buffer {
            if data != **buffer {
                **buffer = data;
                buffer.sync(context);
            }
        } else {
            self.buffer = Some(DynamicBuffer::new(
                data,
                DynamicBufferUsage::Uniform,
                format!("modor_uniform_buffer_material_{}", &self.key.label()),
                &context.device,
            ));
        }
    }
}

impl Resource for Material {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if let Some(error) = &self.error {
            ResourceState::Error(error)
        } else if self.buffer.is_some() && self.bind_group.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::Loading
        }
    }
}

/// A component to update [`Material`] using a component implementing [`MaterialSource`].
///
/// # Requirements
///
/// The material is effective only if:
/// - graphics [`module`](crate::module()) is initialized
/// - the entity contains components of type [`Material`] and `S`
///
/// # Related components
///
/// - [`Material`]
///
/// # Entity functions creating this component
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
/// - [`material`](crate::material())
///
/// # Examples
///
/// See [`instance_group_2d`](crate::instance_group_2d()),
/// [`instance_2d`](crate::instance_2d()) and [`material`](crate::material())
/// as most of the time these methods will be used to create a material.
#[derive(Component, Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct MaterialSync<S: Any> {
    phantom: PhantomData<fn(S)>,
}

#[systems]
impl<S> MaterialSync<S>
where
    S: MaterialSource,
{
    #[run_as(action(MaterialUpdate))]
    fn update_material(
        material: &mut Material,
        source: &S,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        shaders: Custom<ResourceAccessor<'_, Shader>>,
        textures: Custom<ResourceAccessor<'_, Texture>>,
    ) {
        material.update(&renderer, shaders, textures, source);
    }

    #[run_as(action(MaterialUpdate))]
    fn update_instance_groups(
        material: &mut Material,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        instance_renderings: Query<'_, &InstanceRendering2D>,
        mut instance_groups: Custom<ResourceAccessorMut<'_, InstanceGroup2D>>,
        mut query: Query<'_, <S::InstanceData as InstanceData>::Query>,
        mut filtered_query: InstanceDataUpdateQuery<'_, S::InstanceData>,
    ) {
        if mem::size_of::<S::InstanceData>() == 0 {
            return;
        }
        if let Some(context) = Renderer::option_state(&renderer, &mut None).context() {
            for rendering in instance_renderings.iter() {
                if rendering.material_key == material.key {
                    if let Some(instance_group) = instance_groups.get_mut(rendering.group_key) {
                        instance_group.update_material_instances::<S::InstanceData>(
                            &mut query,
                            &mut filtered_query,
                            context,
                        );
                    }
                }
            }
        }
    }
}

#[derive(Action)]
pub(crate) struct MaterialUpdate(
    <Renderer as ComponentSystems>::Action,
    <Shader as ComponentSystems>::Action,
    <ShaderRegistry as ComponentSystems>::Action,
    <Texture as ComponentSystems>::Action,
    <TextureRegistry as ComponentSystems>::Action,
    <InstanceGroup2D as ComponentSystems>::Action,
    <InstanceGroup2DRegistry as ComponentSystems>::Action,
);
