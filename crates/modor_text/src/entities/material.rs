use crate::Text;
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics::{Material, Size, Texture};
use modor_resources::ResKey;

/// Creates a text material entity.
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
///     text_material(MATERIAL, "my text", 30.)
///         .updated(|t: &mut Text| t.font_key = FONT)
///         .updated(|m: &mut Material| m.color = Color::GREEN)         // background color
///         .updated(|m: &mut Material| m.front_color = Color::BLACK)   // text color
///         .component(Model::rectangle(MATERIAL, CAMERA))
///         .component(Transform2D::new())
/// }
/// ```
pub fn text_material(
    material_key: ResKey<Material>,
    text: impl Into<String>,
    font_height: f32,
) -> impl BuiltEntity {
    let texture_key = ResKey::unique("text-material(modor_text)");
    EntityBuilder::new()
        .component(Material::new(material_key))
        .with(|m| m.front_texture_key = Some(texture_key))
        .component(Texture::from_size(texture_key, Size::ZERO))
        .component(Text::new(text, font_height))
}
