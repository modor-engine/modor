use crate::pong::side::Side;
use crate::pong::wall::FIELD_SIZE;
use modor::{App, FromApp, State};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_text::Text2D;

pub(crate) struct Scores {
    pub(crate) is_reset_required: bool,
    is_just_updated: bool,
    left_score: Text2D,
    right_score: Text2D,
}

impl FromApp for Scores {
    fn from_app(app: &mut App) -> Self {
        Self {
            is_reset_required: false,
            is_just_updated: false,
            left_score: Self::score_text(app, Side::Left),
            right_score: Self::score_text(app, Side::Right),
        }
    }
}

impl State for Scores {
    fn update(&mut self, app: &mut App) {
        // `is_reset_required` ensures that all states see this variable equal to `true` at
        // least once. This is not guaranteed for `is_just_updated` depending on state update order.
        if self.is_just_updated {
            self.is_reset_required = true;
            self.is_just_updated = false;
        } else {
            self.is_reset_required = false;
        }
        self.left_score.update(app);
        self.right_score.update(app);
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

    fn score_text(app: &mut App, side: Side) -> Text2D {
        Text2D::new(app)
            .with_model(|m| m.position.x = side.x_sign() * FIELD_SIZE.x / 4.)
            .with_model(|m| m.position.y = FIELD_SIZE.y / 2. - Self::TEXT_HEIGHT / 2.)
            .with_model(|m| m.size = Vec2::new(0.3, Self::TEXT_HEIGHT))
            .with_content("0".into())
    }
}
