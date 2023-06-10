use modor::{systems, App, BuiltEntity, Component, EntityBuilder, Single};
use modor_graphics_new2::{Camera2D, Color, Model, RenderTarget, Window};
use modor_input::{InputModule, Key, Keyboard};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{Alignment, Font, FontSource, Text, TextMaterialBuilder};

pub fn main() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(modor_text::module())
        .with_entity(InputModule::build())
        .with_entity(window())
        .with_entity(Font::new(
            "FontKey",
            FontSource::Path("LuckiestGuy-Regular.ttf".into()),
        ))
        .with_entity(text())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new("TargetKey"))
        .with(Window::default().with_cursor_shown(false))
        .with(Camera2D::new("CameraKey").with_target_key("TargetKey"))
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.8, 0.5)))
        .with(Model::rectangle("MaterialKey::Text").with_camera_key("CameraKey"))
        .with_inherited(
            TextMaterialBuilder::new("MaterialKey::Text", "Hello", 300.)
                .with_text(|t| t.with_font("FontKey").with_alignment(Alignment::Center))
                .with_material(|t| {
                    t.with_color(Color::rgb(0.1, 0.1, 0.1))
                        .with_front_color(Color::WHITE)
                })
                .build(),
        )
        .with(EditableText::default())
}

#[derive(Component, Default)]
struct EditableText {
    is_line_break_char_detected: bool,
}

#[systems]
impl EditableText {
    #[run]
    fn update(&mut self, text: &mut Text, keyboard: Single<'_, Keyboard>) {
        for char in keyboard.text().chars() {
            if char == char::from(13) {
                self.is_line_break_char_detected = true;
                text.content += "\n";
            } else {
                text.content += &char.to_string();
            }
        }
        if !self.is_line_break_char_detected && keyboard.key(Key::Return).is_just_released {
            text.content += "\n";
        }
    }
}
