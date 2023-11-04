use crate::pong::collisions::WALL_GROUP;
use modor::{BuiltEntity, Component, EntityBuilder, NoSystem};
use modor_graphics::{model_2d, Model2DMaterial, WINDOW_CAMERA_2D};
use modor_math::Vec2;
use modor_physics::{Collider2D, Transform2D};

pub(crate) const SIZE: Vec2 = Vec2::new(1. - BORDER_WIDTH, 0.75);
pub(crate) const BORDER_WIDTH: f32 = 0.02;

pub(crate) fn field() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(wall(WallOrientation::Left))
        .child_entity(wall(WallOrientation::Right))
        .child_entity(wall(WallOrientation::Top))
        .child_entity(wall(WallOrientation::Bottom))
        .child_entity(separator())
}

fn wall(orientation: WallOrientation) -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.position = orientation.position())
        .updated(|t: &mut Transform2D| t.size = orientation.size())
        .component(Collider2D::rectangle(WALL_GROUP))
        .component(orientation)
}

fn separator() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(BORDER_WIDTH / 4., SIZE.y))
}

#[derive(Component, NoSystem, Clone, Copy, Debug)]
pub(crate) enum WallOrientation {
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
