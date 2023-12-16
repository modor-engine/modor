use crate::components::material::internal::MaterialData;
use crate::components::renderer::GpuContext;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{
    errors, AnimatedMaterialSource, Color, Renderer, Texture, DEFAULT_SHADER, ELLIPSE_SHADER,
};
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
    <Shader as ComponentSystems>::Action,
    <ShaderRegistry as ComponentSystems>::Action,
    <Texture as ComponentSystems>::Action,
    <TextureRegistry as ComponentSystems>::Action,
);

/// A trait for defining a component type used to configure a [`Material`].
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// # use modor_resources::*;
/// #
/// const BLUR_SHADER: ResKey<Shader> = ResKey::new("blur");
/// const TEXTURE: ResKey<Texture> = ResKey::new("sprite");
///
/// # pub fn no_run() {
/// App::new()
///     .with_entity(modor_graphics::module())
///     .with_entity(window_target())
///     .with_entity(Shader::from_path(BLUR_SHADER, "blur.wgsl"))
///     .with_entity(Texture::from_path(TEXTURE, "texture.png"))
///     .with_entity(sprite())
///     .run(modor_graphics::runner);
/// # }
///
/// fn sprite() -> impl BuiltEntity {
///     let material = BlurMaterial {
///         blur_factor: 0.005,
///         sample_count: 5,
///     };
///     instance_2d(WINDOW_CAMERA_2D, material)
/// }
///
/// #[derive(Component, NoSystem)]
/// struct BlurMaterial {
///     blur_factor: f32,
///     sample_count: u32,
/// }
///
/// impl MaterialSource for BlurMaterial {
///     type Data = BlurMaterialData;
///
///     fn data(&self) -> Self::Data {
///         BlurMaterialData {
///             blur_factor: self.blur_factor,
///             sample_count: self.sample_count,
///             padding: [0., 0.],
///         }
///     }
///
///     fn texture_keys(&self) -> Vec<ResKey<Texture>> {
///         vec![TEXTURE]
///     }
///
///     fn shader_key(&self) -> ResKey<Shader> {
///         BLUR_SHADER
///     }
///
///     fn is_transparent(&self) -> bool {
///         false
///     }
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
/// struct BlurMaterialData {
///     blur_factor: f32,
///     sample_count: u32,
///     padding: [f32; 2],
/// }
/// ```
///
/// where `blur.wgsl` is:
/// ```wgsl
/// struct Camera {
///     transform: mat4x4<f32>,
/// };
///
/// struct Material {
///     blur_factor: f32,
///     sample_count: u32,
///     padding: vec2<f32>,
/// };
///
/// struct Vertex {
///     @location(0)
///     position: vec3<f32>,
///     @location(1)
///     texture_position: vec2<f32>,
/// };
///
/// struct Instance {
///     @location(2)
///     transform_0: vec4<f32>,
///     @location(3)
///     transform_1: vec4<f32>,
///     @location(4)
///     transform_2: vec4<f32>,
///     @location(5)
///     transform_3: vec4<f32>,
/// };
///
/// struct Fragment {
///     @builtin(position)
///     position: vec4<f32>,
///     @location(0)
///     texture_position: vec2<f32>,
/// };
///
/// @group(0)
/// @binding(0)
/// var<uniform> camera: Camera;
///
/// @group(1)
/// @binding(0)
/// var<uniform> material: Material;
///
/// @group(1)
/// @binding(1)
/// var texture: texture_2d<f32>;
///
/// @group(1)
/// @binding(2)
/// var texture_sampler: sampler;
///
/// @vertex
/// fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
///     let transform = mat4x4<f32>(
///         instance.transform_0,
///         instance.transform_1,
///         instance.transform_2,
///         instance.transform_3,
///     );
///     return Fragment(
///         camera.transform * transform * vec4<f32>(vertex.position, 1.),
///         vertex.texture_position,
///     );
/// }
///
/// @fragment
/// fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
///     var color_sum = vec4<f32>();
///     let sample_count_f32 = f32(material.sample_count);
///     for (var x: f32 = -1. * sample_count_f32; x < 1. * sample_count_f32 + 0.5; x += 1.) {
///         for (var y: f32 = -1. * sample_count_f32; y < 1. * sample_count_f32 + 0.5; y += 1.) {
///             color_sum += color(fragment, material.blur_factor * x, material.blur_factor * y);
///         }
///     }
///     return color_sum / pow(sample_count_f32 * 2. + 1., 2.);
/// }
///
/// fn color(fragment: Fragment, offset_x: f32, offset_y: f32) -> vec4<f32> {
///     let texture_position = fragment.texture_position + vec2<f32>(offset_x, offset_y);
///     return textureSample(texture, texture_sampler, texture_position);
/// }
/// ```
pub trait MaterialSource: ComponentSystems {
    /// Raw material data type.
    type Data: Pod + VariableSync + VariableSend;

    /// Returns the raw material data sent to the shader.
    ///
    /// # Platform-specific
    ///
    /// - Web: data size in bytes should be a multiple of 16.
    fn data(&self) -> Self::Data;

    /// Returns the texture keys that are sent to the shader.
    ///
    /// The number of textures should correspond to the number of textures defined in the shader.
    fn texture_keys(&self) -> Vec<ResKey<Texture>>;

    /// Returns the key of the shader used to make the rendering.
    fn shader_key(&self) -> ResKey<Shader>;

    /// Returns whether the rendered instances can be transparent.
    ///
    /// In case `true` is returned, the instances will be rendered in `ZIndex` order.
    /// This is less efficient than for opaque instances, but this limits the risk of having
    /// rendering artifacts caused by transparency.
    ///
    /// Note that transparency is automatically detected for textures returned by
    /// [`MaterialSource::texture_keys`].
    /// It means that if [`MaterialSource::is_transparent`]
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
/// See [`instance_group_2d`](crate::instance_group_2d()),
/// [`instance_2d`](crate::instance_2d()) and [`material`](crate::material())
/// as most of the time these methods will be used to create a material.
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

impl Default2DMaterial {
    /// Creates a new material.
    pub fn new() -> Self {
        Self::default()
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

impl AnimatedMaterialSource for Default2DMaterial {
    fn update(&mut self, sprite_size: Vec2, sprite_position: Vec2) {
        self.texture_size = sprite_size;
        self.texture_position = sprite_position;
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
