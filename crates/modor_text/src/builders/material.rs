use crate::Text;
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics::{Material, Size, Texture};
use modor_resources::IntoResourceKey;

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
/// #
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with_child(Font::from_path(FontKey, "font.ttf"))
///         .with_child(text())
/// }
///
/// fn text() -> impl BuiltEntity {
///     TextMaterialBuilder::new(MaterialKey, "my text", 30.)
///         .with_text(|t| t.with_font(FontKey))
///         .with_material(|m| m.with_color(Color::GREEN))       // background color
///         .with_material(|m| m.with_front_color(Color::BLACK)) // text color
///         .build()
///         .with(Model::rectangle(MaterialKey, CameraKey))
///         .with(Transform2D::new())
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct CameraKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct FontKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct MaterialKey;
/// ```
pub struct TextMaterialBuilder<K> {
    material_key: K,
    text: String,
    font_height: f32,
    material_fn: Option<Box<dyn FnOnce(Material) -> Material>>,
    texture_fn: Option<Box<dyn FnOnce(Texture) -> Texture>>,
    text_fn: Option<Box<dyn FnOnce(Text) -> Text>>,
}

impl<K> TextMaterialBuilder<K>
where
    K: IntoResourceKey + Clone,
{
    /// Creates a new text material builder.
    pub fn new(material_key: K, text: impl Into<String>, font_height: f32) -> Self {
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
        let material = Material::new(self.material_key.clone())
            .with_front_texture_key(self.material_key.clone());
        let texture = Texture::from_size(self.material_key, Size::ZERO);
        let text = Text::new(self.text, self.font_height);
        EntityBuilder::new()
            .with(if let Some(material_fn) = self.material_fn {
                material_fn(material)
            } else {
                material
            })
            .with(if let Some(texture_fn) = self.texture_fn {
                texture_fn(texture)
            } else {
                texture
            })
            .with(if let Some(text_fn) = self.text_fn {
                text_fn(text)
            } else {
                text
            })
    }
}
