use crate::{Button, ButtonStyle, WidgetState};
use modor::BuiltEntity;
use modor_graphics::Camera2D;
use modor_resources::ResKey;
use modor_text::text_2d;

pub fn button(
    camera_key: ResKey<Camera2D>,
    style_key: ResKey<ButtonStyle>,
    text: impl Into<String>,
) -> impl BuiltEntity {
    text_2d(camera_key, text, 0.)
        .component(Button::new(style_key))
        .component(WidgetState::new(camera_key))
}
