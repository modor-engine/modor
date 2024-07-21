use modor::{App, GlobRef, Node, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::Sprite2D;
use modor_physics::{Body2D, CollisionGroupGlob};

pub(crate) const FIELD_BORDER_WIDTH: f32 = 0.02;
pub(crate) const FIELD_SIZE: Vec2 = Vec2::new(1. - FIELD_BORDER_WIDTH, 0.75);

#[derive(Node, Visit)]
pub(crate) struct Wall {
    body: Body2D,
    sprite: Sprite2D,
}

impl Wall {
    pub(crate) fn new(
        app: &mut App,
        orientation: WallOrientation,
        group: GlobRef<CollisionGroupGlob>,
    ) -> Self {
        let body = Body2D::new(app)
            .with_collision_group(Some(group))
            .with_position(orientation.position())
            .with_size(orientation.size());
        Self {
            sprite: Sprite2D::new(app).with_model(|m| m.body = Some(body.glob().to_ref())),
            body,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum WallOrientation {
    Left,
    Right,
    Top,
    Bottom,
}

impl WallOrientation {
    fn position(self) -> Vec2 {
        match self {
            Self::Left => Vec2::X * -FIELD_SIZE.x / 2.,
            Self::Right => Vec2::X * FIELD_SIZE.x / 2.,
            Self::Top => Vec2::Y * FIELD_SIZE.y / 2.,
            Self::Bottom => Vec2::Y * -FIELD_SIZE.y / 2.,
        }
    }

    fn size(self) -> Vec2 {
        match self {
            Self::Left | Self::Right => {
                Vec2::new(FIELD_BORDER_WIDTH, FIELD_SIZE.y + FIELD_BORDER_WIDTH)
            }
            Self::Top | Self::Bottom => Vec2::new(FIELD_SIZE.x, FIELD_BORDER_WIDTH),
        }
    }
}
