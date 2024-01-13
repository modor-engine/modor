#![allow(clippy::trailing_empty_array)]

use crate::components::material_source::internal::Default2DMaterialData;
use crate::components::texture::WHITE_TEXTURE;
use crate::entities::module::{DEFAULT_SHADER, ELLIPSE_SHADER};
use crate::{AnimatedMaterialSource, Color, Shader, Texture};
use bytemuck::Pod;
use modor::{
    ComponentSystems, ConstSystemParam, EntityFilter, QuerySystemParam,
    QuerySystemParamWithLifetime, SystemParamWithLifetime, VariableSend, VariableSync,
};
use modor_math::Vec2;
use modor_resources::ResKey;

/// A trait for defining a component type used to configure a [`Material`](crate::Material).
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
///     .with_entity(Shader::from_path::<BlurInstanceData>(
///         BLUR_SHADER,
///         "blur.wgsl",
///         false,
///     ))
///     .with_entity(Texture::from_path(TEXTURE, "texture.png"))
///     .with_entity(sprite_group())
///     .with_entity(sprite(Vec2::new(-0.25, 0.25), 0))
///     .with_entity(sprite(Vec2::new(0.25, 0.25), 3))
///     .with_entity(sprite(Vec2::new(-0.25, -0.25), 6))
///     .with_entity(sprite(Vec2::new(0.25, -0.25), 9))
///     .run(modor_graphics::runner);
/// # }
///
/// fn sprite_group() -> impl BuiltEntity {
///     instance_group_2d::<With<Blur>>(WINDOW_CAMERA_2D, BlurMaterial { blur_factor: 0.005 })
/// }
///
/// fn sprite(position: Vec2, sample_count: u32) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .with(|t| t.position = position)
///         .with(|t| t.size = Vec2::ONE * 0.4)
///         .component(Blur { sample_count })
/// }
///
/// #[derive(Component, NoSystem)]
/// struct Blur {
///     sample_count: u32,
/// }
///
/// #[derive(Component, NoSystem)]
/// struct BlurMaterial {
///     blur_factor: f32,
/// }
///
/// impl MaterialSource for BlurMaterial {
///     type Data = BlurMaterialData;
///     type InstanceData = BlurInstanceData;
///
///     fn data(&self) -> Self::Data {
///         BlurMaterialData {
///             blur_factor: self.blur_factor,
///             padding1: [0.],
///             padding2: [0., 0.],
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
///     padding1: [f32; 1],
///     padding2: [f32; 2],
/// }
///
/// #[repr(C)]
/// #[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
/// struct BlurInstanceData {
///     sample_count: u32,
/// }
///
/// impl InstanceData for BlurInstanceData {
///     type Query = &'static Blur;
///     type UpdateFilter = Changed<Blur>;
///
///     fn data(item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self {
///         Self {
///             sample_count: item.sample_count,
///         }
///     }
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
///     padding1: f32,        
///     padding2: vec2<f32>,
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
/// struct MaterialInstance {
///     @location(6)
///     sample_count: u32,
/// };
///
/// struct Fragment {
///     @builtin(position)
///     position: vec4<f32>,
///     @location(0)
///     texture_position: vec2<f32>,
///     @location(1)
///     sample_count: u32,
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
/// fn vs_main(vertex: Vertex, instance: Instance, material_instance: MaterialInstance) -> Fragment {
///     let transform = mat4x4<f32>(
///         instance.transform_0,
///         instance.transform_1,
///         instance.transform_2,
///         instance.transform_3,
///     );
///     return Fragment(
///         camera.transform * transform * vec4<f32>(vertex.position, 1.),
///         vertex.texture_position,
///         material_instance.sample_count,
///     );
/// }
///
/// @fragment
/// fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
///     var color_sum = vec4<f32>();
///     let sample_count_f32 = f32(fragment.sample_count);
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
    /// Raw instance data type.
    ///
    /// In case this type has a size of zero with [`mem::size_of`](std::mem::size_of()) (e.g. [`NoInstanceData`]),
    /// then no instance data are sent to the shader.
    ///
    /// It must be the same instance data type as the one passed to [`Shader`].
    type InstanceData: InstanceData;

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
    /// [`MaterialSource::texture_keys`] (except if [`Shader::is_alpha_replaced`] returns `true`).
    ///
    /// It means that if [`MaterialSource::is_transparent`] returns `false` but one of the textures contains
    /// transparent pixels, then the instances are considered as transparent.
    fn is_transparent(&self) -> bool;
}

/// A trait for defining an instance data type.
///
/// # Examples
///
/// See [`MaterialSource`].
pub trait InstanceData: Pod + Default {
    /// Query performed to retrieve entity data needed to construct the instance data.
    type Query: 'static
        + QuerySystemParam
        + for<'a> QuerySystemParamWithLifetime<'a>
        + ConstSystemParam;
    /// Filter on entities that needs to be updated.
    ///
    /// This filter is used to limit performance impact of the instance data update.
    /// In case all instances need to be updated at every frame, `()` update filter can be used.
    type UpdateFilter: EntityFilter;

    /// Constructs instance data using entity data.
    fn data(item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self;
}

/// An empty instance data type.
///
/// Can be used for [`MaterialSource::InstanceData`] if no material instance data need to be sent to the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct NoInstanceData;

impl InstanceData for NoInstanceData {
    type Query = ();
    type UpdateFilter = ();

    // coverage: off (method never called)
    fn data(_item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self {
        unreachable!("disabled instance data")
    }
    // coverage: on
}

/// The default material configuration for 2D rendering.
///
/// # Requirements
///
/// The material is effective only if:
/// - graphics [`module`](crate::module()) is initialized
/// - the entity contains components of type [`Material`](crate::Material)
///     and [`MaterialSync<Default2DMaterial>`](crate::MaterialSync)
///
/// # Related components
///
/// - [`Material`](crate::Material)
/// - [`MaterialSync`](crate::MaterialSync)
/// - [`Texture`]
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
    type Data = Default2DMaterialData;
    type InstanceData = NoInstanceData;

    fn data(&self) -> Self::Data {
        Default2DMaterialData {
            color: self.color.into(),
            texture_position: [self.texture_position.x, self.texture_position.y],
            texture_size: [self.texture_size.x, self.texture_size.y],
        }
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![self.texture_key.unwrap_or(WHITE_TEXTURE)]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        if self.is_ellipse {
            ELLIPSE_SHADER
        } else {
            DEFAULT_SHADER
        }
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
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
    pub struct Default2DMaterialData {
        pub(crate) color: [f32; 4],
        pub(crate) texture_position: [f32; 2],
        pub(crate) texture_size: [f32; 2],
    }
}
