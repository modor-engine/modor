use crate::Text;
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics::{Material, Size, Texture};
use modor_resources::ResKey;

/// A builder for constructing an entity with a text [`Material`].
///
/// The created entity contains the following components:
/// - [`Text`]
/// - [`Texture`]
/// - [`Material`]
///
/// # Requirements
///
/// - text [`module`](crate::module()) is initialized
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_physics::*;
/// # use modor_text::*;
/// # use modor_resources::*;
/// #
/// const CAMERA: ResKey<Camera2D> = ResKey::new("main");
/// const FONT: ResKey<Font> = ResKey::new("custom");
/// const MATERIAL: ResKey<Material> = ResKey::new("text");
///
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .child_component(Font::from_path(FONT, "font.ttf"))
///         .child_entity(text())
/// }
///
/// fn text() -> impl BuiltEntity {
///     TextMaterialBuilder::new(MATERIAL, "my text", 30.)
///         .with_text(|t| t.with_font(FONT))
///         .with_material(|m| m.with_color(Color::GREEN))       // background color
///         .with_material(|m| m.with_front_color(Color::BLACK)) // text color
///         .build()
///         .component(Model::rectangle(MATERIAL, CAMERA))
///         .component(Transform2D::new())
/// }
/// ```
pub struct TextMaterialBuilder {
    material_key: ResKey<Material>,
    text: String,
    font_height: f32,
    material_fn: Option<Box<dyn FnOnce(Material) -> Material>>,
    texture_fn: Option<Box<dyn FnOnce(Texture) -> Texture>>,
    text_fn: Option<Box<dyn FnOnce(Text) -> Text>>,
}

impl TextMaterialBuilder {
    /// Creates a new text material builder.
    pub fn new(material_key: ResKey<Material>, text: impl Into<String>, font_height: f32) -> Self {
        Self {
            material_key,
            text: text.into(),
            font_height,
            material_fn: None,
            texture_fn: None,
            text_fn: None,
        }
    }

    /// Overrides the configuration of the created [`Material`] component.
    ///
    /// If this method is called more than once, only the last call is taken into account.
    pub fn with_material(mut self, f: impl FnOnce(Material) -> Material + 'static) -> Self {
        self.material_fn = Some(Box::new(f));
        self
    }

    /// Overrides the configuration of the created [`Texture`] component.
    ///
    /// If this method is called more than once, only the last call is taken into account.
    pub fn with_texture(mut self, f: impl FnOnce(Texture) -> Texture + 'static) -> Self {
        self.texture_fn = Some(Box::new(f));
        self
    }

    /// Overrides the configuration of the created [`Text`] component.
    ///
    /// If this method is called more than once, only the last call is taken into account.
    pub fn with_text(mut self, f: impl FnOnce(Text) -> Text + 'static) -> Self {
        self.text_fn = Some(Box::new(f));
        self
    }

    /// Builds the entity.
    pub fn build(self) -> impl BuiltEntity {
        let texture_key = ResKey::unique("text-material(modor_text)");
        let material = Material::new(self.material_key).with_front_texture_key(texture_key);
        let texture = Texture::from_size(texture_key, Size::ZERO);
        let text = Text::new(self.text, self.font_height);
        EntityBuilder::new()
            .component(if let Some(material_fn) = self.material_fn {
                material_fn(material)
            } else {
                material
            })
            .component(if let Some(texture_fn) = self.texture_fn {
                texture_fn(texture)
            } else {
                texture
            })
            .component(if let Some(text_fn) = self.text_fn {
                text_fn(text)
            } else {
                text
            })
    }
}
