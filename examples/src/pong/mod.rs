use crate::pong::ball::Ball;
use crate::pong::paddle::Paddle;
use crate::pong::scores::Scores;
use crate::pong::wall::{FIELD_BORDER_WIDTH, FIELD_SIZE};
use collisions::CollisionGroups;
use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::Sprite2D;
use modor_physics::modor_math::Vec2;
use side::Side;
use wall::{Wall, WallOrientation};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    left_wall: Wall,
    right_wall: Wall,
    top_wall: Wall,
    bottom_wall: Wall,
    separator: Sprite2D,
    ball: Ball,
    left_paddle: Paddle,
    right_paddle: Paddle,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.create::<Scores>();
        let groups = ctx.get_mut::<CollisionGroups>();
        let vertical_wall_group = groups.vertical_wall.glob().clone();
        let horizontal_wall_group = groups.horizontal_wall.glob().clone();
        Self {
            left_wall: Wall::new(ctx, WallOrientation::Left, vertical_wall_group.clone()),
            right_wall: Wall::new(ctx, WallOrientation::Right, vertical_wall_group),
            top_wall: Wall::new(ctx, WallOrientation::Top, horizontal_wall_group.clone()),
            bottom_wall: Wall::new(ctx, WallOrientation::Bottom, horizontal_wall_group),
            separator: Sprite2D::new(ctx, "separator")
                .with_model(|m| m.size = Vec2::new(FIELD_BORDER_WIDTH / 4., FIELD_SIZE.y)),
            ball: Ball::new(ctx),
            left_paddle: Paddle::new_player(ctx, Side::Left),
            right_paddle: Paddle::new_bot(ctx, Side::Right),
        }
    }
}

mod ball;
mod collisions;
mod paddle;
mod scores;
mod side;
mod wall;
