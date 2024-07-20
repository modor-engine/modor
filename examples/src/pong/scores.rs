use crate::pong::side::Side;
use crate::pong::wall::FIELD_SIZE;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_text::Text2D;

#[derive(Visit)]
pub(crate) struct Scores {
    pub(crate) is_reset_required: bool,
    is_just_updated: bool,
    left_score: Text2D,
    right_score: Text2D,
}

impl RootNode for Scores {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            is_reset_required: false,
            is_just_updated: false,
            left_score: Self::score_text(ctx, Side::Left),
            right_score: Self::score_text(ctx, Side::Right),
        }
    }
}

impl Node for Scores {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        // `is_reset_required` ensures that all nodes see this variable equal to `true` at
        // least once. This is not guaranteed for `is_just_updated` depending on node update order.
        if self.is_just_updated {
            self.is_reset_required = true;
            self.is_just_updated = false;
        } else {
            self.is_reset_required = false;
        }
    }
}

impl Scores {
    const TEXT_HEIGHT: f32 = 0.2;

    pub(crate) fn increment(&mut self, side: Side) {
        if self.is_reset_required {
            return;
        }
        let score = match side {
            Side::Left => &mut self.left_score,
            Side::Right => &mut self.right_score,
        };
        score.content = (score
            .content
            .parse::<u64>()
            .expect("internal error: invalid score")
            + 1)
        .to_string();
        self.is_just_updated = true;
    }

    fn score_text(ctx: &mut Context<'_>, side: Side) -> Text2D {
        Text2D::new(ctx, "score")
            .with_model(|m| m.position.x = side.x_sign() * FIELD_SIZE.x / 4.)
            .with_model(|m| m.position.y = FIELD_SIZE.y / 2. - Self::TEXT_HEIGHT / 2.)
            .with_model(|m| m.size = Vec2::new(0.3, Self::TEXT_HEIGHT))
            .with_content("0".into())
    }
}
