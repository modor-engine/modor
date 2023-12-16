use modor::{systems, App, BuiltEntity, SingleRef, SingletonComponent};
use modor_graphics::{window_target, Color, Default2DMaterial, WINDOW_CAMERA_2D};
use modor_input::Keyboard;
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{text_2d, Text};

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(text(0.375, "Last entered text:"))
        .with_entity(text(0.125, "").component(EnteredText::default()))
        .with_entity(text(-0.125, "Pressed keys:"))
        .with_entity(text(-0.375, "").component(PressedKeys::default()))
        .run(modor_graphics::runner);
}

fn text(position_y: f32, text: &str) -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, text.to_string(), 50.)
        .updated(|t: &mut Transform2D| t.position = Vec2::Y * position_y)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(1., 0.15))
        .updated(|m: &mut Default2DMaterial| m.color = Color::INVISIBLE)
        .updated(|m: &mut Default2DMaterial| m.front_color = Color::WHITE)
}

#[derive(SingletonComponent, Default)]
struct EnteredText(String);

#[systems]
impl EnteredText {
    #[run_after(component(Keyboard))]
    fn retrieve(&mut self, keyboard: SingleRef<'_, '_, Keyboard>) {
        let new_text = keyboard.get().text.clone();
        if !new_text.is_empty() {
            self.0 = new_text;
        }
    }

    #[run_after_previous]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.clone();
    }
}

#[derive(SingletonComponent, Default)]
struct PressedKeys(Vec<String>);

#[systems]
impl PressedKeys {
    #[run_after(component(Keyboard))]
    fn retrieve(&mut self, keyboard: SingleRef<'_, '_, Keyboard>) {
        self.0.clear();
        for key in keyboard.get().pressed_iter() {
            self.0.push(format!("{key:?}"));
        }
    }

    #[run_after_previous]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.join(", ");
    }
}
