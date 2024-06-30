use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::Inputs;
use modor_physics::modor_math::Vec2;
use modor_text::Text2D;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Visit)]
struct Root {
    last_entered_text_label: Text2D,
    last_entered_text: Text2D,
    pressed_keys_label: Text2D,
    pressed_keys: Text2D,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            last_entered_text_label: text(ctx, 0.375, "Last entered text:"),
            last_entered_text: text(ctx, 0.125, ""),
            pressed_keys_label: text(ctx, -0.125, "Pressed keys:"),
            pressed_keys: text(ctx, -0.375, ""),
        }
    }
}

impl Node for Root {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let keyboard = &ctx.get_mut::<Inputs>().keyboard;
        if !keyboard.text.is_empty() {
            self.last_entered_text.content.clone_from(&keyboard.text);
        }
        self.pressed_keys.content = keyboard
            .pressed_iter()
            .map(|key| format!("{key:?}"))
            .collect::<Vec<_>>()
            .join(", ");
    }
}

fn text(ctx: &mut Context<'_>, position_y: f32, content: &str) -> Text2D {
    Text2D::new(ctx, "text")
        .with_model(|m| m.position = Vec2::Y * position_y)
        .with_model(|m| m.size = Vec2::new(1., 0.15))
        .with_content(content.into())
        .with_font_height(50.)
}
