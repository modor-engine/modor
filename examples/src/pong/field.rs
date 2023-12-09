use crate::pong::collisions::{HORIZONTAL_WALL_GROUP, LEFT_WALL_GROUP, RIGHT_WALL_GROUP};
use modor::{BuiltEntity, Component, EntityBuilder, NoSystem};
use modor_graphics::{instance_2d, WINDOW_CAMERA_2D};
use modor_math::Vec2;
use modor_physics::{Collider2D, CollisionGroup, Transform2D};
use modor_resources::ResKey;

pub(crate) const SIZE: Vec2 = Vec2::new(1. - BORDER_WIDTH, 0.75);
pub(crate) const BORDER_WIDTH: f32 = 0.02;

pub(crate) fn field() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(wall(WallOrientation::Left, LEFT_WALL_GROUP))
        .child_entity(wall(WallOrientation::Right, RIGHT_WALL_GROUP))
        .child_entity(wall(WallOrientation::Top, HORIZONTAL_WALL_GROUP))
        .child_entity(wall(WallOrientation::Bottom, HORIZONTAL_WALL_GROUP))
        .child_entity(separator())
}

fn wall(orientation: WallOrientation, group_key: ResKey<CollisionGroup>) -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|t: &mut Transform2D| t.position = orientation.position())
        .updated(|t: &mut Transform2D| t.size = orientation.size())
        .component(Collider2D::rectangle(group_key))
        .component(orientation)
}

fn separator() -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(BORDER_WIDTH / 4., SIZE.y))
}

#[derive(Component, NoSystem, Clone, Copy, Debug)]
enum WallOrientation {
    Left,
    Right,
    Top,
    Bottom,
}

impl WallOrientation {
    fn position(self) -> Vec2 {
        match self {
            Self::Left => Vec2::X * -SIZE.x / 2.,
            Self::Right => Vec2::X * SIZE.x / 2.,
            Self::Top => Vec2::Y * SIZE.y / 2.,
            Self::Bottom => Vec2::Y * -SIZE.y / 2.,
        }
    }

    fn size(self) -> Vec2 {
        match self {
            Self::Left | Self::Right => Vec2::new(BORDER_WIDTH, SIZE.y + BORDER_WIDTH),
            Self::Top | Self::Bottom => Vec2::new(SIZE.x, BORDER_WIDTH),
        }
    }
}
