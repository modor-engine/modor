use crate::pong::collisions::{BALL_GROUP, PADDLE_GROUP};
use crate::pong::events::ResetEvent;
use crate::pong::field::WallOrientation;
use crate::pong::paddles;
use crate::pong::scores::{LeftScore, RightScore};
use instant::Instant;
use modor::{systems, BuiltEntity, Query, SingleMut, SingleRef, SingletonComponent, World};
use modor_graphics::{model_2d, Model2DMaterial, WINDOW_CAMERA_2D};
use modor_math::Vec2;
use modor_physics::{Collider2D, Dynamics2D, Transform2D};
use rand::Rng;
use std::f32::consts::FRAC_PI_4;

const SIZE: Vec2 = Vec2::new(0.03, 0.03);
const INITIAL_SPEED: f32 = 0.6;
const ACCELERATION: f32 = 0.05;

pub(crate) fn ball() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.position = Vec2::ZERO)
        .updated(|t: &mut Transform2D| t.size = SIZE)
        .component(Dynamics2D::new())
        .with(|d| d.velocity = generate_ball_velocity())
        .component(Collider2D::circle(BALL_GROUP))
        .component(Ball::default())
}

#[derive(SingletonComponent, Debug)]
pub(crate) struct Ball {
    creation_instant: Instant,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            creation_instant: Instant::now(),
        }
    }
}

#[systems]
impl Ball {
    #[run_after(component(Transform2D), component(Dynamics2D))]
    fn reset(
        &mut self,
        transform: &mut Transform2D,
        dynamics: &mut Dynamics2D,
        _event: SingleRef<'_, '_, ResetEvent>,
    ) {
        self.creation_instant = Instant::now();
        transform.position = Vec2::ZERO;
        dynamics.velocity = generate_ball_velocity();
    }

    #[run_after(component(Dynamics2D), component(Collider2D))]
    fn handle_collision_with_walls(
        dynamics: &mut Dynamics2D,
        collider: &mut Collider2D,
        walls: Query<'_, &WallOrientation>,
        reset_event: Option<SingleRef<'_, '_, ResetEvent>>,
        mut left_score: SingleMut<'_, '_, LeftScore>,
        mut right_score: SingleMut<'_, '_, RightScore>,
        mut world: World<'_>,
    ) {
        for (_, wall) in collider.collided(&walls) {
            match wall {
                WallOrientation::Left => {
                    if reset_event.is_none() {
                        world.create_root_entity(ResetEvent);
                        right_score.get_mut().0 += 1;
                    }
                }
                WallOrientation::Right => {
                    if reset_event.is_none() {
                        world.create_root_entity(ResetEvent);
                        left_score.get_mut().0 += 1;
                    }
                }
                WallOrientation::Top => dynamics.velocity.y = -dynamics.velocity.y.abs(),
                WallOrientation::Bottom => dynamics.velocity.y = dynamics.velocity.y.abs(),
            }
        }
    }

    #[run_after(component(Dynamics2D), component(Transform2D), component(Collider2D))]
    fn handle_collision_with_paddle(
        dynamics: &mut Dynamics2D,
        transform: &Transform2D,
        collider: &Collider2D,
        transforms: Query<'_, &Transform2D>,
    ) {
        for (_, paddle_transform) in collider.collided_as(&transforms, PADDLE_GROUP) {
            let normalized_direction = -transform.position.x.signum();
            let direction = dynamics.velocity.magnitude() * normalized_direction;
            let relative_y_offset = normalized_direction
                * (transform.position.y - paddle_transform.position.y)
                / (paddles::SIZE.y / 2.);
            let rotation = relative_y_offset * FRAC_PI_4;
            dynamics.velocity = Vec2::new(direction, 0.).with_rotation(rotation);
        }
    }

    #[run_after(component(Dynamics2D))]
    fn update_acceleration(&self, dynamics: &mut Dynamics2D) {
        let speed = self
            .creation_instant
            .elapsed()
            .as_secs_f32()
            .mul_add(ACCELERATION, INITIAL_SPEED);
        dynamics.velocity = dynamics
            .velocity
            .with_magnitude(speed)
            .expect("ball velocity is zero");
    }
}

fn generate_ball_velocity() -> Vec2 {
    let mut rng = rand::thread_rng();
    let direction = if rng.gen_bool(0.5) { -1. } else { 1. };
    Vec2::new(direction * INITIAL_SPEED, 0.)
}
