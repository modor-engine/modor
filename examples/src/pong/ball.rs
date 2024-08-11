use crate::pong::collisions::CollisionGroups;
use crate::pong::paddle::Paddle;
use crate::pong::scores::Scores;
use crate::pong::side::Side;
use instant::Instant;
use modor::{App, FromApp, Glob, Globals, State, StateHandle};
use modor_graphics::Sprite2D;
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, Body2DUpdater};
use rand::Rng;
use std::f32::consts::FRAC_PI_4;

pub(crate) struct Ball {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    init_instant: Instant,
    collision_groups: StateHandle<CollisionGroups>,
    bodies: StateHandle<Globals<Body2D>>,
}

impl FromApp for Ball {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            init_instant: Instant::now(),
            collision_groups: app.handle(),
            bodies: app.handle(),
        }
    }
}

impl Ball {
    const SIZE: Vec2 = Vec2::new(0.03, 0.03);
    const INITIAL_SPEED: f32 = 0.6;
    const ACCELERATION: f32 = 0.05;

    pub(crate) fn init(&mut self, app: &mut App) {
        Body2DUpdater::default()
            .position(Vec2::ZERO)
            .size(Self::SIZE)
            .velocity(Self::generate_velocity())
            .mass(1.)
            .is_ccd_enabled(true)
            .collision_group(self.collision_groups.get(app).ball.to_ref())
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
        self.sprite.material.is_ellipse = true;
        self.init_instant = Instant::now();
    }

    pub(crate) fn update(&mut self, app: &mut App) {
        self.handle_collision_with_paddle(app);
        self.handle_collision_with_ball(app);
        self.apply_acceleration(app);
        self.reset_on_score(app);
        self.sprite.update(app);
        app.get_mut::<BallProperties>().position = self.body.get(app).position(app);
    }

    fn generate_velocity() -> Vec2 {
        let mut rng = rand::thread_rng();
        let direction = if rng.gen_bool(0.5) { -1. } else { 1. };
        Vec2::new(direction * Self::INITIAL_SPEED, 0.)
    }

    pub(crate) fn handle_collision_with_paddle(&mut self, app: &mut App) {
        let Some(paddle) = self.collided_paddle(app) else {
            return;
        };
        let body = self.body.get(app);
        let normalized_direction = -body.position(app).x.signum();
        let direction = body.velocity(app).magnitude() * normalized_direction;
        let relative_y_offset = normalized_direction
            * (body.position(app).y - paddle.position(app).y)
            / (Paddle::SIZE.y / 2.);
        let rotation = relative_y_offset * FRAC_PI_4;
        Body2DUpdater::default()
            .velocity(Vec2::new(direction, 0.).with_rotation(rotation))
            .apply(app, &self.body);
    }

    pub(crate) fn handle_collision_with_ball(&mut self, app: &mut App) {
        let body = self.body.get(app);
        let position = body.position(app);
        let vertical_wall_group = &self.collision_groups.get(app).vertical_wall;
        if body.is_colliding_with(vertical_wall_group) {
            app.get_mut::<Scores>().increment(if position.x < 0. {
                Side::Right
            } else {
                Side::Left
            });
        }
    }

    pub(crate) fn reset_on_score(&mut self, app: &mut App) {
        if app.get_mut::<Scores>().is_reset_required {
            self.init(app);
        }
    }

    fn apply_acceleration(&mut self, app: &mut App) {
        let speed = self
            .init_instant
            .elapsed()
            .as_secs_f32()
            .mul_add(Self::ACCELERATION, Self::INITIAL_SPEED);
        Body2DUpdater::default()
            .for_velocity(|v| {
                *v = v
                    .with_magnitude(speed)
                    .expect("internal error: ball velocity is zero");
            })
            .apply(app, &self.body);
    }

    fn collided_paddle<'a>(&self, app: &'a App) -> Option<&'a Body2D> {
        let paddle_group = &self.collision_groups.get(app).paddle;
        let collision = self.body.get(app).collisions_with(paddle_group).next()?;
        self.bodies.get(app).get(collision.other_index)
    }
}

#[derive(Default, State)]
pub(crate) struct BallProperties {
    pub(crate) position: Vec2,
}
