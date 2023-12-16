use crate::Text;
use modor::BuiltEntity;
use modor_graphics::{instance_2d, Camera2D, Default2DMaterial, Size, Texture};
use modor_resources::ResKey;

/// Creates a 2D text entity.
///
/// The created entity contains the following components:
/// - All components created by [`instance_2d`](instance_2d::<Default2DMaterial>()), including [`Material`]
/// - [`Text`]
/// - [`Texture`]
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
///     text_2d(CAMERA, "my text", 30.)
///         .updated(|t: &mut Text| t.font_key = FONT)
///         .updated(|m: &mut Default2DMaterial| m.color = Color::GREEN)         // background color
///         .updated(|m: &mut Default2DMaterial| m.front_color = Color::BLACK)   // text color
/// }
/// ```
pub fn text_2d(
    camera_key: ResKey<Camera2D>,
    text: impl Into<String>,
    font_height: f32,
) -> impl BuiltEntity {
    let texture_key = ResKey::unique("text-2d(modor_text)");
    instance_2d::<Default2DMaterial>(camera_key, None)
        .updated(|m: &mut Default2DMaterial| m.front_texture_key = Some(texture_key))
        .component(Texture::from_size(texture_key, Size::ZERO))
        .component(Text::new(text, font_height))
}
