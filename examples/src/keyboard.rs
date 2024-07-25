use modor::log::Level;
use modor::{App, FromApp, RootNode};
use modor_graphics::modor_input::Inputs;
use modor_physics::modor_math::Vec2;
use modor_text::Text2D;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    last_entered_text_label: Text2D,
    last_entered_text: Text2D,
    pressed_keys_label: Text2D,
    pressed_keys: Text2D,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            last_entered_text_label: text(app, 0.375, "Last entered text:"),
            last_entered_text: text(app, 0.125, ""),
            pressed_keys_label: text(app, -0.125, "Pressed keys:"),
            pressed_keys: text(app, -0.375, ""),
        }
    }
}

impl RootNode for Root {
    fn update(&mut self, app: &mut App) {
        let keyboard = &app.get_mut::<Inputs>().keyboard;
        if !keyboard.text.is_empty() {
            self.last_entered_text.content.clone_from(&keyboard.text);
        }
        self.pressed_keys.content = keyboard
            .pressed_iter()
            .map(|key| format!("{key:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        self.last_entered_text_label.update(app);
        self.last_entered_text.update(app);
        self.pressed_keys_label.update(app);
        self.pressed_keys.update(app);
    }
}

fn text(app: &mut App, position_y: f32, content: &str) -> Text2D {
    Text2D::new(app)
        .with_model(|m| m.position = Vec2::Y * position_y)
        .with_model(|m| m.size = Vec2::new(1., 0.15))
        .with_content(content.into())
        .with_font_height(50.)
}
