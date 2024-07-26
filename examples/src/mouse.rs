use modor::log::Level;
use modor::{App, FromApp, State};
use modor_graphics::modor_input::Inputs;
use modor_graphics::{CursorTracker, Sprite2D};
use modor_physics::modor_math::Vec2;
use modor_text::Text2D;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    pressed_buttons_label: Text2D,
    pressed_buttons: Text2D,
    cursor: Sprite2D,
    tracker: CursorTracker,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            pressed_buttons_label: text(app, 0.25, "Pressed buttons:"),
            pressed_buttons: text(app, -0.25, ""),
            cursor: Sprite2D::new(app)
                .with_model(|m| m.size = Vec2::ONE * 0.02)
                .with_material(|m| m.is_ellipse = true),
            tracker: CursorTracker::new(app),
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        let mouse = &app.get_mut::<Inputs>().mouse;
        self.pressed_buttons.content = mouse
            .pressed_iter()
            .map(|button| format!("{button:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        self.tracker.update(app);
        self.cursor.model.position = self.tracker.position(app);
        self.pressed_buttons_label.update(app);
        self.pressed_buttons.update(app);
        self.cursor.update(app);
    }
}

fn text(app: &mut App, position_y: f32, content: &str) -> Text2D {
    Text2D::new(app)
        .with_model(|m| m.position = Vec2::Y * position_y)
        .with_model(|m| m.size = Vec2::new(1., 0.15))
        .with_content(content.into())
        .with_font_height(50.)
}
