#![allow(missing_docs)]

use modor::{systems, App, BuiltEntity, SingleRef, SingletonComponent};
use modor_graphics::{window_target, Color, Material, WINDOW_CAMERA_2D};
use modor_input::{GamepadStick, Gamepads};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{text_2d, Text};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(text(0.375, "Moved sticks:"))
        .with_entity(text(0.125, "").component(MovedSticks::default()))
        .with_entity(text(-0.125, "Pressed buttons:"))
        .with_entity(text(-0.375, "").component(PressedButtons::default()))
        .run(modor_graphics::runner);
}

fn text(position_y: f32, text: &str) -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, text.to_string(), 50.)
        .updated(|t: &mut Transform2D| *t.position = Vec2::Y * position_y)
        .updated(|t: &mut Transform2D| *t.size = Vec2::new(1., 0.15))
        .updated(|t: &mut Material| t.color = Color::INVISIBLE)
        .updated(|t: &mut Material| t.front_color = Color::WHITE)
}

#[derive(SingletonComponent, Default)]
struct MovedSticks(Vec<String>);

#[systems]
impl MovedSticks {
    #[run_after(component(Gamepads))]
    fn retrieve(&mut self, gamepads: SingleRef<'_, '_, Gamepads>) {
        let gamepads = gamepads.get();
        self.0.clear();
        if let Some(gamepad_id) = gamepads.iter().next() {
            let gamepad = &gamepads[gamepad_id];
            if gamepad[GamepadStick::LeftStick] != Vec2::ZERO {
                self.0.push("LeftStick".into());
            }
            if gamepad[GamepadStick::RightStick] != Vec2::ZERO {
                self.0.push("RightStick".into());
            }
            if gamepad[GamepadStick::DPad] != Vec2::ZERO {
                self.0.push("DPad".into());
            }
        }
    }

    #[run_after_previous]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.join(", ");
    }
}

#[derive(SingletonComponent, Default)]
struct PressedButtons(Vec<String>);

#[systems]
impl PressedButtons {
    #[run_after(component(Gamepads))]
    fn retrieve(&mut self, gamepads: SingleRef<'_, '_, Gamepads>) {
        let gamepads = gamepads.get();
        self.0.clear();
        if let Some(gamepad_id) = gamepads.iter().next() {
            let gamepad = &gamepads[gamepad_id];
            for button in gamepad.pressed_iter() {
                self.0.push(format!("{button:?}"));
            }
        }
    }

    #[run_after_previous]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.join(", ");
    }
}
