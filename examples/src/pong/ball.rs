use crate::pong::collisions::CollisionGroups;
use crate::pong::paddle::Paddle;
use crate::pong::scores::Scores;
use crate::pong::side::Side;
use instant::Instant;
use modor::{App, Globals, Node, RootNode, Visit};
use modor_graphics::Sprite2D;
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, Body2DGlob};
use rand::Rng;
use std::f32::consts::FRAC_PI_4;

#[derive(Visit)]
pub(crate) struct Ball {
    body: Body2D,
    sprite: Sprite2D,
    creation_instant: Instant,
}

impl Node for Ball {
    fn on_enter(&mut self, app: &mut App) {
        self.body.update(app); // to use the latest state of the body
        self.handle_collision_with_paddle(app);
        self.handle_collision_with_ball(app);
        self.apply_acceleration();
        self.reset_on_score(app);
    }

    fn on_exit(&mut self, app: &mut App) {
        app.get_mut::<BallProperties>().position = self.body.position;
    }
}

impl Ball {
    const SIZE: Vec2 = Vec2::new(0.03, 0.03);
    const INITIAL_SPEED: f32 = 0.6;
    const ACCELERATION: f32 = 0.05;

    pub(crate) fn new(app: &mut App) -> Self {
        let group = app.get_mut::<CollisionGroups>().ball.glob().clone();
        let body = Body2D::new(app)
            .with_position(Vec2::ZERO)
            .with_size(Self::SIZE)
            .with_velocity(Self::generate_velocity())
            .with_mass(1.)
            .with_is_ccd_enabled(true)
            .with_collision_group(Some(group));
        Self {
            sprite: Sprite2D::new(app)
                .with_model(|m| m.body = Some(body.glob().clone()))
                .with_material(|m| m.is_ellipse = true),
            body,
            creation_instant: Instant::now(),
        }
    }

    fn generate_velocity() -> Vec2 {
        let mut rng = rand::thread_rng();
        let direction = if rng.gen_bool(0.5) { -1. } else { 1. };
        Vec2::new(direction * Self::INITIAL_SPEED, 0.)
    }

    pub(crate) fn handle_collision_with_paddle(&mut self, app: &mut App) {
        let paddle_group = app.get_mut::<CollisionGroups>().paddle.glob();
        let Some(collision) = self.body.collisions_with(paddle_group).next() else {
            return;
        };
        let paddle = &app.get_mut::<Globals<Body2DGlob>>()[collision.other_index];
        let normalized_direction = -self.body.position.x.signum();
        let direction = self.body.velocity.magnitude() * normalized_direction;
        let relative_y_offset = normalized_direction * (self.body.position.y - paddle.position.y)
            / (Paddle::SIZE.y / 2.);
        let rotation = relative_y_offset * FRAC_PI_4;
        self.body.velocity = Vec2::new(direction, 0.).with_rotation(rotation);
    }

    pub(crate) fn handle_collision_with_ball(&mut self, app: &mut App) {
        let vertical_wall_group = app.get_mut::<CollisionGroups>().vertical_wall.glob();
        if self.body.is_colliding_with(vertical_wall_group) {
            app.get_mut::<Scores>()
                .increment(if self.body.position.x < 0. {
                    Side::Right
                } else {
                    Side::Left
                });
        }
    }

    pub(crate) fn reset_on_score(&mut self, app: &mut App) {
        if app.get_mut::<Scores>().is_reset_required {
            self.body.position = Vec2::ZERO;
            self.body.velocity = Self::generate_velocity();
            self.creation_instant = Instant::now();
        }
    }

    fn apply_acceleration(&mut self) {
        let speed = self
            .creation_instant
            .elapsed()
            .as_secs_f32()
            .mul_add(Self::ACCELERATION, Self::INITIAL_SPEED);
        self.body.velocity = self
            .body
            .velocity
            .with_magnitude(speed)
            .expect("internal error: ball velocity is zero");
    }
}

#[derive(Default, RootNode, Node, Visit)]
pub(crate) struct BallProperties {
    pub(crate) position: Vec2,
}
