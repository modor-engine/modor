use modor::{App, FromApp, Glob};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::Sprite2D;
use modor_physics::{Body2D, Body2DUpdater, CollisionGroup};

pub(crate) const FIELD_BORDER_WIDTH: f32 = 0.02;
pub(crate) const FIELD_SIZE: Vec2 = Vec2::new(1. - FIELD_BORDER_WIDTH, 0.75);

pub(crate) struct Wall {
    body: Glob<Body2D>,
    sprite: Sprite2D,
}

impl FromApp for Wall {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
        }
    }
}

impl Wall {
    pub(crate) fn init(
        &mut self,
        app: &mut App,
        orientation: WallOrientation,
        group: &Glob<CollisionGroup>,
    ) {
        Body2DUpdater::default()
            .position(orientation.position())
            .size(orientation.size())
            .collision_group(group.to_ref())
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
    }

    pub(crate) fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
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
