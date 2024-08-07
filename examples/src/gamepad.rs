use modor::log::Level;
use modor::{App, FromApp, State};
use modor_graphics::modor_input::{GamepadStick, Inputs};
use modor_physics::modor_math::Vec2;
use modor_text::Text2D;

const STICK_LABELS: [(GamepadStick, &str); 3] = [
    (GamepadStick::LeftStick, "LeftStick"),
    (GamepadStick::RightStick, "RightStick"),
    (GamepadStick::DPad, "DPad"),
];

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    moved_sticks_label: Text2D,
    moved_sticks: Text2D,
    pressed_buttons_label: Text2D,
    pressed_buttons: Text2D,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            moved_sticks_label: text(app, 0.375, "Moved sticks:"),
            moved_sticks: text(app, 0.125, ""),
            pressed_buttons_label: text(app, -0.125, "Pressed buttons:"),
            pressed_buttons: text(app, -0.375, ""),
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        let gamepads = &app.get_mut::<Inputs>().gamepads;
        if let Some((_, gamepad)) = gamepads.iter().next() {
            self.moved_sticks.content = STICK_LABELS
                .into_iter()
                .filter(|(stick, _)| gamepad[*stick] != Vec2::ZERO)
                .map(|(_, label)| label)
                .collect::<Vec<_>>()
                .join(", ");
            self.pressed_buttons.content = gamepad
                .pressed_iter()
                .map(|button| format!("{button:?}"))
                .collect::<Vec<_>>()
                .join(", ");
        } else {
            self.moved_sticks.content.clear();
            self.pressed_buttons.content.clear();
        }
        self.moved_sticks_label.update(app);
        self.moved_sticks.update(app);
        self.pressed_buttons_label.update(app);
        self.pressed_buttons.update(app);
    }
}

fn text(app: &mut App, position_y: f32, content: &str) -> Text2D {
    Text2D::new(app)
        .with_model(|m| m.position = Vec2::Y * position_y)
        .with_model(|m| m.size = Vec2::new(1., 0.15))
        .with_content(content.into())
        .with_font_height(50.)
}
