use crate::pong::ball::Ball;
use crate::pong::collisions::PADDLE_GROUP;
use crate::pong::events::ResetEvent;
use crate::pong::Side;
use modor::{systems, BuiltEntity, Component, Single, SingleRef};
use modor_graphics::{instance_2d, Camera2D, Window, WINDOW_CAMERA_2D};
use modor_input::{Fingers, Key, Keyboard};
use modor_math::Vec2;
use modor_physics::{Collider2D, Dynamics2D, Transform2D};

pub(crate) const SIZE: Vec2 = Vec2::new(0.04, 0.18);
const SPEED: f32 = 1.;

pub(crate) fn player_paddle(side: Side) -> impl BuiltEntity {
    let player = match side {
        Side::Left => PaddlePlayer {
            up_key: Key::ArrowUp,
            down_key: Key::ArrowDown,
            touch_min_x: -1.,
            touch_max_x: 0.,
        },
        Side::Right => PaddlePlayer {
            up_key: Key::ArrowUp,
            down_key: Key::ArrowDown,
            touch_min_x: 0.,
            touch_max_x: 1.,
        },
    };
    paddle(side).component(player)
}

pub(crate) fn bot_paddle(side: Side) -> impl BuiltEntity {
    paddle(side).component(PaddleBot)
}

fn paddle(side: Side) -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|t: &mut Transform2D| t.position = Vec2::X * 0.4 * side.x_sign())
        .updated(|t: &mut Transform2D| t.size = SIZE)
        .component(Dynamics2D::new())
        .with(|d| d.mass = 1.)
        .component(Collider2D::rectangle(PADDLE_GROUP))
        .component(Paddle)
}

#[derive(Component, Debug)]
pub(crate) struct Paddle;

#[systems]
impl Paddle {
    #[run_after(component(Transform2D))]
    fn reset(transform: &mut Transform2D, _event: SingleRef<'_, '_, ResetEvent>) {
        transform.position.y = 0.;
    }
}

#[derive(Component, Debug)]
struct PaddlePlayer {
    up_key: Key,
    down_key: Key,
    touch_min_x: f32,
    touch_max_x: f32,
}

#[systems]
impl PaddlePlayer {
    #[run_after(
        component(Dynamics2D),
        component(Transform2D),
        component(Keyboard),
        component(Fingers),
        component(Window),
        component(Camera2D)
    )]
    fn update_velocity(
        &self,
        dynamics: &mut Dynamics2D,
        transform: &Transform2D,
        keyboard: SingleRef<'_, '_, Keyboard>,
        fingers: SingleRef<'_, '_, Fingers>,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
    ) {
        let fingers = fingers.get();
        dynamics.velocity.y = SPEED * keyboard.get().axis(self.down_key, self.up_key);
        if dynamics.velocity == Vec2::ZERO {
            for finger_id in fingers.iter() {
                let (window, camera) = window_camera.get();
                let position = camera.world_position(window.size(), fingers[finger_id].position);
                if position.x >= self.touch_min_x && position.x <= self.touch_max_x {
                    dynamics.velocity.y = paddle_speed(transform, position, 0.01);
                    break;
                }
            }
        }
    }
}

#[derive(Component, Debug)]
struct PaddleBot;

#[systems]
impl PaddleBot {
    #[run_after(component(Dynamics2D), component(Transform2D), component(Ball))]
    fn update_velocity(
        dynamics: &mut Dynamics2D,
        transform: &Transform2D,
        ball_transform: Single<'_, Ball, &Transform2D>,
    ) {
        dynamics.velocity.y = paddle_speed(transform, ball_transform.get().position, 0.1);
    }
}

fn paddle_speed(paddle_transform: &Transform2D, objective_position: Vec2, precision: f32) -> f32 {
    let objective_paddle_diff_y = objective_position.y - paddle_transform.position.y;
    if objective_paddle_diff_y > precision {
        SPEED
    } else if objective_paddle_diff_y < -precision {
        -SPEED
    } else {
        0.
    }
}
