use crate::components::material::internal::MaterialData;
use crate::components::renderer::GpuContext;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{errors, Color, Renderer, Texture, TextureAnimation, DEFAULT_SHADER, ELLIPSE_SHADER};
use bytemuck::Pod;
use derivative::Derivative;
use modor::{Component, ComponentSystems, Custom, SingleRef, VariableSend, VariableSync};
use modor_math::Vec2;
use modor_resources::{
    ResKey, Resource, ResourceAccessor, ResourceLoadingError, ResourceRegistry, ResourceState,
};
use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;
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
/// - [`InstanceRendering2D`](crate::InstanceRendering2D)
///
/// # Entity functions creating this component
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
/// - [`material`](crate::material())
///
/// # Examples
///
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
#[must_use]
#[derive(Component, Debug)]
pub struct Material {
    pub(crate) shader_key: ResKey<Shader>,
    pub(crate) is_transparent: bool,
    pub(crate) bind_group: Option<BindGroup>,
    pub(crate) texture_keys: Vec<ResKey<Texture>>,
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
            key,
            buffer: None,
            renderer_version: None,
            error: None,
        }
    }

    fn update(
        &mut self,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        shaders: Custom<ResourceAccessor<'_, Shader>>,
        textures: Custom<ResourceAccessor<'_, Texture>>,
        source: &impl MaterialSource,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
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
                self.is_transparent =
                    source.is_transparent() || textures.iter().any(|t| t.inner().is_transparent);
                self.texture_keys = texture_keys;
                self.shader_key = shader_key;
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
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
#[derive(Component, Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct MaterialSync<S: Any> {
    phantom: PhantomData<fn(S)>,
}

#[systems]
impl<S> MaterialSync<S>
where
    S: ComponentSystems + MaterialSource,
{
    #[run_as(action(MaterialUpdate))]
    fn update(
        material: &mut Material,
        source: &S,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        shaders: Custom<ResourceAccessor<'_, Shader>>,
        textures: Custom<ResourceAccessor<'_, Texture>>,
    ) {
        material.update(renderer, shaders, textures, source);
    }
}

#[derive(Action)]
pub(crate) struct MaterialUpdate(
    <Renderer as ComponentSystems>::Action,
    <TextureAnimation as ComponentSystems>::Action,
    <Shader as ComponentSystems>::Action,
    <ShaderRegistry as ComponentSystems>::Action,
    <Texture as ComponentSystems>::Action,
    <TextureRegistry as ComponentSystems>::Action,
);

/// A trait for defining a component type used to configure a [`Material`].
///
/// # Example
///
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
pub trait MaterialSource {
    /// Raw material data type.
    type Data: Pod + VariableSync + VariableSend + Debug;

    /// Returns the raw material data sent to the shader.
    fn data(&self) -> Self::Data;

    /// Returns the texture keys that are sent to the shader.
    ///
    /// The number of textures should correspond to the number of textures defined in the shader.
    fn texture_keys(&self) -> Vec<ResKey<Texture>>;

    /// Returns the key of the shader used to make the rendering.
    fn shader_key(&self) -> ResKey<Shader>;

    // specify that texture transparent is automatically detected
    /// Returns whether the rendered instances can be transparent.
    ///
    /// In case `true` is returned, the instances will be rendered in `ZIndex` order.
    /// This is less efficient than for opaque instances, but this limits the risk of having
    /// rendering artifacts caused by transparency.
    ///
    /// Note that transparency is automatically detected for textures returned by
    /// [`MaterialSource::texture_key`]. It means that if [`MaterialSource::is_transparent`]
    /// returns `false` but one of the textures contains transparent pixels, then the instances
    /// are considered as transparent.
    fn is_transparent(&self) -> bool;
}

/// The default material configuration for 2D rendering.
///
/// # Requirements
///
/// The material is effective only if:
/// - graphics [`module`](crate::module()) is initialized
/// - the entity contains components of type [`Material`] and [`MaterialSync<Default2DMaterial>`]
///
/// # Related components
///
/// - [`Material`]
/// - [`MaterialSync`]
///
/// # Examples
///
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
#[derive(Component, NoSystem, Debug)]
pub struct Default2DMaterial {
    /// Color of the rendered instance.
    ///
    /// This color is multiplied to the texture when a [`texture_key`](#structfield.texture_key)
    /// is defined.
    ///
    /// Default is [`Color::WHITE`].
    pub color: Color,
    /// Key of the [`Texture`] used to render the instance.
    ///
    /// If the texture is not loaded, then the instances attached to the material are not rendered.
    ///
    /// Default is [`None`].
    pub texture_key: Option<ResKey<Texture>>,
    /// Top-left position of the extracted texture section.
    ///
    /// [`Vec2::ZERO`] corresponds to top-left corner, and [`Vec2::ONE`] corresponds to bottom-right
    /// corner of the texture.
    ///
    /// Default is [`Vec2::ZERO`].
    pub texture_position: Vec2,
    /// Size of the extracted texture section.
    ///
    /// [`Vec2::ONE`] corresponds to the entire texture.
    ///
    /// Default is [`Vec2::ONE`].
    pub texture_size: Vec2,
    /// Key of the foreground texture.
    ///
    /// This texture is placed on top of the main texture defined using
    /// [`texture_key`](#structfield.texture_key). In contrary to the main texture, the initial
    /// aspect ratio is always kept during rendering. For example with a rectangle instance:
    /// - Main texture is stretched to cover the whole rectangle, so the aspect ratio might not be
    /// kept.
    /// - Foreground texture is centered on the rectangle and keeps its aspect ratio,
    /// which means the texture might not cover the whole rectangle.
    ///
    /// For example, the foreground texture is useful for rendering a text that should not be
    /// stretched.
    ///
    /// If the texture is not loaded, then the instances attached to the material are not rendered.
    ///
    /// Default is [`None`].
    pub front_texture_key: Option<ResKey<Texture>>,
    /// Color that is multiplied to the foreground texture when
    /// [`front_texture_key`](#structfield.front_texture_key) is defined.
    ///
    /// Default is [`Color::BLACK`].
    pub front_color: Color,
    /// Whether the instance is rendered as an ellipse.
    ///
    /// If `false`, then the instance is displayed as a rectangle.
    ///
    /// Default is `false`.
    pub is_ellipse: bool,
}

impl Default for Default2DMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture_key: None,
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            front_texture_key: None,
            front_color: Color::BLACK,
            is_ellipse: false,
        }
    }
}

impl MaterialSource for Default2DMaterial {
    type Data = MaterialData;

    fn data(&self) -> Self::Data {
        MaterialData {
            color: self.color.into(),
            texture_part_position: [self.texture_position.x, self.texture_position.y],
            texture_part_size: [self.texture_size.x, self.texture_size.y],
            front_color: self.front_color.into(),
        }
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![
            self.texture_key.unwrap_or(WHITE_TEXTURE),
            self.front_texture_key.unwrap_or(INVISIBLE_TEXTURE),
        ]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        if self.is_ellipse {
            ELLIPSE_SHADER
        } else {
            DEFAULT_SHADER
        }
    }

    fn is_transparent(&self) -> bool {
        (self.color.a > 0. && self.color.a < 1.)
            || (self.front_color.a > 0. && self.front_color.a < 1.)
    }
}

mod internal {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct MaterialData {
        pub(crate) color: [f32; 4],
        pub(crate) texture_part_position: [f32; 2],
        pub(crate) texture_part_size: [f32; 2],
        pub(crate) front_color: [f32; 4],
    }
}
