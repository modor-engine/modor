use modor::{systems, App, BuiltEntity, Single, SingleMut, SingleRef, SingletonComponent};
use modor_graphics::{window_target, Camera2D, Color, Material, Window, WINDOW_CAMERA_2D};
use modor_input::{Fingers, Keyboard, Mouse, MouseButton, VirtualKeyboard};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{text_2d, Text};

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(text(0.375, "Last entered text:"))
        .with_entity(text(0.200, "").component(EnteredText::default()))
        .with_entity(text(-0.200, "Pressed keys:"))
        .with_entity(text(-0.375, "").component(PressedKeys::default()))
        .with_entity(virtual_keyboard_toggle_button())
        .run(modor_graphics::runner);
}

fn text(position_y: f32, text: &str) -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, text.to_string(), 50.)
        .updated(|t: &mut Transform2D| t.position = Vec2::Y * position_y)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(1., 0.15))
        .updated(|t: &mut Material| t.color = Color::INVISIBLE)
        .updated(|t: &mut Material| t.front_color = Color::WHITE)
}

fn virtual_keyboard_toggle_button() -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, "Open virtual keyboard", 50.)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.6, 0.1))
        .component(VirtualKeyboardToggleButton)
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

#[derive(SingletonComponent)]
struct VirtualKeyboardToggleButton;

#[systems]
impl VirtualKeyboardToggleButton {
    #[run_after(component(VirtualKeyboard))]
    fn update(
        text: &mut Text,
        mut virtual_keyboard: SingleMut<'_, '_, VirtualKeyboard>,
        mouse: SingleRef<'_, '_, Mouse>,
        fingers: SingleRef<'_, '_, Fingers>,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
    ) {
        let mouse = mouse.get();
        let fingers = fingers.get();
        if !Self::is_cursor_just_released(mouse, fingers) {
            return;
        }
        let (window, camera) = window_camera.get();
        let cursor_position = Self::cursor_position(mouse, fingers);
        let cursor_position = camera.world_position(window.size(), cursor_position);
        if Self::is_cursor_on_button(cursor_position) {
            let virtual_keyboard = virtual_keyboard.get_mut();
            if virtual_keyboard.is_open() {
                virtual_keyboard.close();
                text.content = "Open virtual keyboard".into();
            } else {
                virtual_keyboard.open();
                text.content = "Close virtual keyboard".into();
            }
        }
    }

    fn is_cursor_just_released(mouse: &Mouse, fingers: &Fingers) -> bool {
        fingers
            .iter()
            .next()
            .map_or(mouse[MouseButton::Left].is_just_released(), |i| {
                fingers[i].state.is_just_released()
            })
    }

    fn cursor_position(mouse: &Mouse, fingers: &Fingers) -> Vec2 {
        fingers
            .iter()
            .next()
            .map_or(mouse.position, |i| fingers[i].position)
    }

    fn is_cursor_on_button(cursor_position: Vec2) -> bool {
        cursor_position.x >= -0.3
            && cursor_position.x <= 0.3
            && cursor_position.y >= -0.05
            && cursor_position.y <= 0.05
    }
}
