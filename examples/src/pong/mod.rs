use crate::pong::ball::ball;
use crate::pong::collisions::collision_groups;
use crate::pong::field::field;
use crate::pong::paddles::{bot_paddle, player_paddle};
use crate::pong::scores::score;
use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics::window_target;

mod ball;
mod collisions;
mod events;
mod field;
mod paddles;
mod scores;

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(collision_groups())
        .with_entity(game())
        .run(modor_graphics::runner);
}

pub(crate) fn game() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(field())
        .child_entity(score(Side::Left))
        .child_entity(score(Side::Right))
        .child_entity(player_paddle(Side::Left))
        .child_entity(bot_paddle(Side::Right))
        .child_entity(ball())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Right,
}

impl Side {
    pub(crate) fn x_sign(self) -> f32 {
        match self {
            Self::Left => -1.,
            Self::Right => 1.,
        }
    }
}
