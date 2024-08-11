use crate::pong::ball::BallProperties;
use crate::pong::collisions::CollisionGroups;
use crate::pong::scores::Scores;
use crate::pong::side::Side;
use modor::{App, FromApp, Glob, StateHandle};
use modor_graphics::modor_input::{Inputs, Key};
use modor_graphics::{Sprite2D, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, Body2DUpdater};

pub(crate) struct Paddle {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    controls: Option<PlayerControls>,
    window: StateHandle<Window>,
    inputs: StateHandle<Inputs>,
}

impl FromApp for Paddle {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            controls: None,
            window: app.handle(),
            inputs: app.handle(),
        }
    }
}

impl Paddle {
    pub(crate) const SIZE: Vec2 = Vec2::new(0.04, 0.18);
    const SPEED: f32 = 0.9;

    pub(crate) fn init_player(&mut self, app: &mut App, side: Side) {
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
        self.init(app, side, Some(controls));
    }

    pub(crate) fn init_bot(&mut self, app: &mut App, side: Side) {
        self.init(app, side, None);
    }

    pub(crate) fn update(&mut self, app: &mut App) {
        let new_velocity = self.new_velocity(app);
        Body2DUpdater::default()
            .for_velocity(|v| v.y = new_velocity)
            .apply(app, &self.body);
        self.reset_on_score(app);
        self.sprite.update(app);
    }

    fn init(&mut self, app: &mut App, side: Side, controller: Option<PlayerControls>) {
        Body2DUpdater::default()
            .position(Vec2::X * 0.4 * side.x_sign())
            .size(Self::SIZE)
            .collision_group(app.get_mut::<CollisionGroups>().paddle.to_ref())
            .mass(1.)
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
        self.controls = controller;
    }

    fn speed(&self, app: &App, objective_position: Vec2, precision: f32) -> f32 {
        let position = self.body.get(app).position(app);
        let objective_paddle_diff_y = objective_position.y - position.y;
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
                    .map(|position| self.speed(app, position, 0.02))
                    .next()
                    .unwrap_or_else(|| self.body.get(app).velocity(app).y)
            } else {
                inputs.keyboard.axis(controls.down_key, controls.up_key) * Self::SPEED
            }
        } else {
            let ball_position = app.get_mut::<BallProperties>().position;
            self.speed(app, ball_position, 0.1)
        }
    }

    fn reset_on_score(&mut self, app: &mut App) {
        if app.get_mut::<Scores>().is_reset_required {
            Body2DUpdater::default()
                .for_position(|p| p.y = 0.)
                .apply(app, &self.body);
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
