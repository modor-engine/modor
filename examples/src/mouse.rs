use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::Inputs;
use modor_graphics::{CursorTracker, Sprite2D};
use modor_physics::modor_math::Vec2;
use modor_text::Text2D;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Visit)]
struct Root {
    pressed_buttons_label: Text2D,
    pressed_buttons: Text2D,
    cursor: Sprite2D,
    tracker: CursorTracker,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            pressed_buttons_label: text(ctx, 0.25, "Pressed buttons:"),
            pressed_buttons: text(ctx, -0.25, ""),
            cursor: Sprite2D::new(ctx, "cursor")
                .with_model(|m| m.size = Vec2::ONE * 0.02)
                .with_material(|m| m.is_ellipse = true),
            tracker: CursorTracker::new(ctx),
        }
    }
}

impl Node for Root {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let mouse = &ctx.get_mut::<Inputs>().mouse;
        self.pressed_buttons.content = mouse
            .pressed_iter()
            .map(|button| format!("{button:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        self.cursor.model.position = self.tracker.position(ctx);
    }
}

fn text(ctx: &mut Context<'_>, position_y: f32, content: &str) -> Text2D {
    Text2D::new(ctx, "text")
        .with_model(|m| m.position = Vec2::Y * position_y)
        .with_model(|m| m.size = Vec2::new(1., 0.15))
        .with_content(content.into())
        .with_font_height(50.)
}
