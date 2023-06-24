use crate::components::shader::ShaderKey;
use crate::components::texture::TextureRegistry;
use crate::gpu_data::uniform::Uniform;
use crate::{Color, Renderer, Texture};
use modor::{Query, Single, SingleMut};
use modor_math::Vec2;
use modor_resources::{IntoResourceKey, Resource, ResourceKey, ResourceRegistry, ResourceState};

pub(crate) type MaterialRegistry = ResourceRegistry<Material>;

/// The aspect of a rendered [`Model`](crate::Model).
///
/// # Requirements
///
/// The material is effective only if:
/// - graphics [`module`](crate::module()) is initialized
///
/// # Related components
///
/// - [`Model`](crate::Model)
///
/// # Performance
///
/// As models are rendered by batch based on the material, recreating the same material for each
/// model is less performant than creating it once for all models.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// # use modor_graphics::*;
/// #
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with_child(Material::ellipse(MaterialKey::BlueEllipse).with_color(Color::BLUE))
///         .with_child(Material::new(MaterialKey::FullTex).with_texture_key(TextureKey))
///         .with_child(texture_quarter_material(MaterialKey::TopLeftTex, Vec2::new(0., 0.)))
///         .with_child(texture_quarter_material(MaterialKey::TopRightTex, Vec2::new(0.5, 0.)))
///         .with_child(texture_quarter_material(MaterialKey::BottomLeftTex, Vec2::new(0., 0.5)))
///         .with_child(texture_quarter_material(MaterialKey::BottomRightTex, Vec2::new(0.5, 0.5)))
///         .with_child(sprite(Vec2::new(0.4, 0.2)))
/// }
///
/// fn sprite(position: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new().with_position(position).with_size(Vec2::new(0.1, 0.1)))
///         .with(Model::rectangle(MaterialKey::TopLeftTex, CameraKey))
/// }
///
/// fn texture_quarter_material(key: MaterialKey, position: Vec2) -> Material {
///     Material::new(key)
///         .with_texture_key(TextureKey)
///         .with_texture_position(position)
///         .with_texture_size(Vec2::new(0.5, 0.5))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum MaterialKey {
///     BlueEllipse,
///     FullTex,
///     TopLeftTex,
///     TopRightTex,
///     BottomLeftTex,
///     BottomRightTex,
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct TextureKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct CameraKey;
/// ```
#[must_use]
#[derive(Component, Debug)]
pub struct Material {
    /// Color of the rendered model.
    ///
    /// This color is multiplied to the texture when a [`texture_key`](#structfield.texture_key)
    /// is defined.
    ///
    /// Default is [`Color::WHITE`].
    pub color: Color,
    /// Key of the [`Texture`] used to render the model.
    ///
    /// If the texture is not loaded, then the models attached to the material are not rendered.
    ///
    /// Default is [`None`].
    pub texture_key: Option<ResourceKey>,
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
    /// aspect ratio is always kept during rendering. For example with a rectangle model:
    /// - Main texture is stretched to cover the whole rectangle, so the aspect ratio might not be
    /// kept.
    /// - Foreground texture is centered on the rectangle and keeps its aspect ratio,
    /// which means the texture might not cover the whole rectangle.
    ///
    /// For example, the foreground texture is useful for rendering a text that should not be
    /// stretched.
    ///
    /// If the texture is not loaded, then the models attached to the material are not rendered.
    ///
    /// Default is [`None`].
    pub front_texture_key: Option<ResourceKey>,
    /// Color that is multiplied to the foreground texture when
    /// [`front_texture_key`](#structfield.front_texture_key) is defined.
    ///
    /// Default is [`Color::BLACK`].
    pub front_color: Color,
    pub(crate) shader_key: ResourceKey,
    key: ResourceKey,
    uniform: Option<Uniform<MaterialData>>,
    is_transparent: bool,
    old_is_transparent: bool,
    renderer_version: Option<u8>,
}

#[systems]
impl Material {
    const BINDING: u32 = 0;

    /// Creates a new material with a unique `key`.
    pub fn new(key: impl IntoResourceKey) -> Self {
        Self::new_internal(key, ShaderKey::Default)
    }

    /// Creates a material with a unique `key` that crops the rendered model to obtain an ellipse.
    pub fn ellipse(key: impl IntoResourceKey) -> Self {
        Self::new_internal(key, ShaderKey::Ellipse)
    }

    /// Returns the material with a given [`color`](#structfield.color).
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the material with a given [`texture_key`](#structfield.texture_key).
    pub fn with_texture_key(mut self, texture_key: impl IntoResourceKey) -> Self {
        self.texture_key = Some(texture_key.into_key());
        self
    }

    /// Returns the material with a given [`texture_position`](#structfield.texture_position).
    pub fn with_texture_position(mut self, texture_position: Vec2) -> Self {
        self.texture_position = texture_position;
        self
    }

    /// Returns the material with a given [`texture_size`](#structfield.texture_size).
    pub fn with_texture_size(mut self, texture_size: Vec2) -> Self {
        self.texture_size = texture_size;
        self
    }

    /// Returns the material with a given [`front_texture_key`](#structfield.front_texture_key).
    pub fn with_front_texture_key(mut self, front_texture_key: impl IntoResourceKey) -> Self {
        self.front_texture_key = Some(front_texture_key.into_key());
        self
    }

    /// Returns the material with a given [`front_color`](#structfield.front_color).
    pub fn with_front_color(mut self, front_color: Color) -> Self {
        self.front_color = front_color;
        self
    }

    fn new_internal(key: impl IntoResourceKey, shader_key: ShaderKey) -> Self {
        Self {
            color: Color::WHITE,
            texture_key: None,
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            front_texture_key: None,
            front_color: Color::BLACK,
            key: key.into_key(),
            shader_key: shader_key.into_key(),
            uniform: None,
            is_transparent: false,
            old_is_transparent: false,
            renderer_version: None,
        }
    }

    #[run_after(component(Renderer))]
    fn update_uniform(&mut self, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.uniform = None;
        }
        if let Some(context) = state.context() {
            let data = MaterialData {
                color: self.color.into(),
                texture_part_position: [self.texture_position.x, self.texture_position.y],
                texture_part_size: [self.texture_size.x, self.texture_size.y],
                front_color: self.front_color.into(),
            };
            if let Some(uniform) = &mut self.uniform {
                if data != **uniform {
                    **uniform = data;
                    uniform.sync(context);
                }
            } else {
                self.uniform = Some(Uniform::new(
                    data,
                    Self::BINDING,
                    &context.material_bind_group_layout,
                    &format!("material_{:?}", &self.key),
                    &context.device,
                ));
            }
        }
    }

    #[run_after(component(TextureRegistry), component(Texture))]
    fn update_transparency(
        &mut self,
        (mut texture_registry, textures): (SingleMut<'_, TextureRegistry>, Query<'_, &Texture>),
    ) {
        self.old_is_transparent = self.is_transparent;
        if !self.is_transparent {
            self.is_transparent = (self.color.a > 0. && self.color.a < 1.)
                || self
                    .texture_key
                    .as_ref()
                    .and_then(|k| texture_registry.get(k, &textures))
                    .map_or(false, |t| t.inner().is_transparent);
        }
    }

    pub(crate) fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub(crate) fn is_newly_transparent(&self) -> bool {
        !self.old_is_transparent && self.is_transparent
    }

    pub(crate) fn uniform(&self) -> &Uniform<MaterialData> {
        self.uniform
            .as_ref()
            .expect("internal error: material uniform not initialized")
    }
}

impl Resource for Material {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.uniform.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::NotLoaded
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct MaterialData {
    pub(crate) color: [f32; 4],
    pub(crate) texture_part_position: [f32; 2],
    pub(crate) texture_part_size: [f32; 2],
    pub(crate) front_color: [f32; 4],
}
