use crate::pong::ball::BallProperties;
use crate::pong::collisions::CollisionGroups;
use crate::pong::scores::Scores;
use crate::pong::side::Side;
use modor::{App, RootNodeHandle};
use modor_graphics::modor_input::{Inputs, Key};
use modor_graphics::{Sprite2D, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::Body2D;

pub(crate) struct Paddle {
    body: Body2D,
    sprite: Sprite2D,
    controls: Option<PlayerControls>,
    window: RootNodeHandle<Window>,
    inputs: RootNodeHandle<Inputs>,
}

impl Paddle {
    pub(crate) const SIZE: Vec2 = Vec2::new(0.04, 0.18);
    const SPEED: f32 = 0.9;

    pub(crate) fn new_player(app: &mut App, side: Side) -> Self {
        let controls = match side {
            Side::Left => PlayerControls {
                up_key: Key::KeyW,
                down_key: Key::KeyS,
                min_touch_zone_x: -1.,
                max_touch_zone_x: 0.,
            },
            Side::Right => PlayerControls {
                up_key: Key::ArrowUp,
                down_key: Key::ArrowDown,
                min_touch_zone_x: 0.,
                max_touch_zone_x: 1.,
            },
        };
        Self::new(app, side, Some(controls))
    }

    pub(crate) fn new_bot(app: &mut App, side: Side) -> Self {
        Self::new(app, side, None)
    }

    fn new(app: &mut App, side: Side, controller: Option<PlayerControls>) -> Self {
        let group = app.get_mut::<CollisionGroups>().paddle.glob().to_ref();
        let body = Body2D::new(app)
            .with_position(Vec2::X * 0.4 * side.x_sign())
            .with_size(Self::SIZE)
            .with_collision_group(Some(group))
            .with_mass(1.);
        Self {
            sprite: Sprite2D::new(app).with_model(|m| m.body = Some(body.glob().to_ref())),
            body,
            controls: controller,
            window: app.handle(),
            inputs: app.handle(),
        }
    }

    pub(crate) fn update(&mut self, app: &mut App) {
        self.body.velocity.y = self.new_velocity(app);
        self.reset_on_score(app);
        self.body.update(app);
        self.sprite.update(app);
    }

    fn speed(&self, objective_position: Vec2, precision: f32) -> f32 {
        let objective_paddle_diff_y = objective_position.y - self.body.position.y;
        if objective_paddle_diff_y > precision {
            Self::SPEED
        } else if objective_paddle_diff_y < -precision {
            -Self::SPEED
        } else {
            0.
        }
    }

    fn new_velocity(&mut self, app: &mut App) -> f32 {
        if let Some(controls) = self.controls {
            let inputs = self.inputs.get(app);
            if inputs.fingers.pressed_iter().count() > 0 {
                let window = self.window.get(app);
                let window_size = window.size();
                let camera = window.camera.glob();
                inputs
                    .fingers
                    .pressed_iter()
                    .map(|(_, finger)| camera.get(app).world_position(window_size, finger.position))
                    .filter(|position| position.x >= controls.min_touch_zone_x)
                    .filter(|position| position.x <= controls.max_touch_zone_x)
                    .map(|position| self.speed(position, 0.02))
                    .next()
                    .unwrap_or(self.body.velocity.y)
            } else {
                inputs.keyboard.axis(controls.down_key, controls.up_key) * Self::SPEED
            }
        } else {
            let ball_position = app.get_mut::<BallProperties>().position;
            self.speed(ball_position, 0.1)
        }
    }

    pub(crate) fn reset_on_score(&mut self, app: &mut App) {
        if app.get_mut::<Scores>().is_reset_required {
            self.body.position.y = 0.;
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PlayerControls {
    up_key: Key,
    down_key: Key,
    min_touch_zone_x: f32,
    max_touch_zone_x: f32,
}
