#![allow(missing_docs)]

use crate::ball::ball;
use crate::field::field;
use crate::paddles::{bot_paddle, player_paddle};
use crate::scores::score;
use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics::window_target;
use modor_input::InputModule;
use modor_physics::{CollisionGroupRef, CollisionType, PhysicsModule};

mod ball;
mod events;
mod field;
mod paddles;
mod scores;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(InputModule::build())
        .with_entity(modor_text::module())
        .with_entity(window_target())
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum CollisionGroup {
    Wall,
    Paddle,
    Ball,
}

impl CollisionGroupRef for CollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Ball, Self::Wall | Self::Paddle) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
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
