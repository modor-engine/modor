use approx::AbsDiffEq;
use instant::Instant;
use modor::log::Level;
use modor::{App, FromApp, Glob, State, StateHandle};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_input::{Inputs, Key};
use modor_graphics::{Color, Sprite2D};
use modor_physics::{Body2D, CollisionGroup, Impulse};
use std::time::Duration;

const PLATFORM_PERIOD: Duration = Duration::from_secs(4);
const CHARACTER_MASS: f32 = 1.;
const GRAVITY_FACTOR: f32 = -2.;
const JUMP_FACTOR: f32 = 50.;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(FromApp)]
struct Root;

impl State for Root {
    fn init(&mut self, app: &mut App) {
        app.create::<Character>();
        app.create::<Platforms>();
    }
}

#[derive(FromApp)]
struct Platforms {
    platforms: Vec<Platform>,
}

impl State for Platforms {
    fn init(&mut self, app: &mut App) {
        self.platforms = vec![
            // ground
            Platform::from_app_with(app, |platform, app| {
                platform.init(app, Vec2::new(0., -0.4), Vec2::new(1., 0.02), Vec2::ZERO);
            }),
            // wall
            Platform::from_app_with(app, |platform, app| {
                platform.init(app, Vec2::new(-0.5, 0.), Vec2::new(0.02, 0.82), Vec2::ZERO);
            }),
            // dynamic platforms
            Platform::from_app_with(app, |platform, app| {
                platform.init(
                    app,
                    Vec2::new(0., 0.2),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(0.15, 0.),
                );
            }),
            Platform::from_app_with(app, |platform, app| {
                platform.init(
                    app,
                    Vec2::new(0., 0.05),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(-0.2, 0.),
                );
            }),
            Platform::from_app_with(app, |platform, app| {
                platform.init(
                    app,
                    Vec2::new(0., -0.1),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(0.05, 0.),
                );
            }),
            Platform::from_app_with(app, |platform, app| {
                platform.init(
                    app,
                    Vec2::new(0., -0.25),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(-0.1, 0.),
                );
            }),
        ];
    }

    fn update(&mut self, app: &mut App) {
        for platform in &mut self.platforms {
            platform.update(app);
        }
    }
}

impl Platforms {
    fn find(&self, body_index: usize) -> Option<&Platform> {
        self.platforms
            .iter()
            .find(|platform| platform.body.index() == body_index)
    }
}

#[derive(FromApp)]
struct CollisionGroups {
    platform: Glob<CollisionGroup>,
    character: Glob<CollisionGroup>,
}

impl State for CollisionGroups {
    fn init(&mut self, app: &mut App) {
        self.character
            .updater()
            .add_impulse(app, &self.platform, Impulse::new(0., 0.));
    }
}

struct Platform {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    next_reverse_instant: Instant,
}

impl FromApp for Platform {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            next_reverse_instant: Instant::now() + PLATFORM_PERIOD,
        }
    }
}

impl Platform {
    fn init(&mut self, app: &mut App, position: Vec2, size: Vec2, velocity: Vec2) {
        self.body
            .updater()
            .position(position)
            .size(size)
            .velocity(velocity)
            .collision_group(app.get_mut::<CollisionGroups>().platform.to_ref())
            .apply(app);
        self.sprite.model.body = Some(self.body.to_ref());
        self.sprite.material.color = Color::GREEN;
    }

    fn update(&mut self, app: &mut App) {
        if Instant::now() >= self.next_reverse_instant {
            self.next_reverse_instant = Instant::now() + PLATFORM_PERIOD;
            self.body
                .updater()
                .for_velocity(app, |velocity| *velocity *= -1.)
                .apply(app);
        }
        self.sprite.update(app);
    }
}

struct Character {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    platforms: StateHandle<Platforms>,
}

impl FromApp for Character {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            platforms: app.handle(),
        }
    }
}

impl State for Character {
    fn init(&mut self, app: &mut App) {
        self.body
            .updater()
            .position(Vec2::new(0., 0.5))
            .size(Vec2::new(0.03, 0.1))
            .collision_group(Some(app.get_mut::<CollisionGroups>().character.to_ref()))
            .mass(CHARACTER_MASS)
            .force(Vec2::Y * GRAVITY_FACTOR * CHARACTER_MASS)
            .apply(app);
        self.sprite.model.body = Some(self.body.to_ref());
    }

    fn update(&mut self, app: &mut App) {
        let keyboard = &app.get_mut::<Inputs>().keyboard;
        let x_movement = keyboard.axis(Key::ArrowLeft, Key::ArrowRight);
        let is_jump_pressed = keyboard[Key::ArrowUp].is_pressed();
        let touched_ground = self.touched_ground(app);
        let ground_velocity =
            touched_ground.map_or(0., |platform| platform.body.get(app).velocity(app).x);
        self.body
            .updater()
            .force(self.force(app, touched_ground.is_some(), is_jump_pressed))
            .for_velocity(app, |v| v.x = 0.5f32.mul_add(x_movement, ground_velocity))
            .apply(app);
        self.sprite.update(app);
    }
}

impl Character {
    fn touched_ground<'a>(&'a self, app: &'a App) -> Option<&Platform> {
        self.body
            .get(app)
            .collisions()
            .iter()
            .filter_map(|collision| self.platforms.get(app).find(collision.other_index))
            .find(|platform| self.is_on_platform(app, platform))
    }

    fn is_on_platform(&self, app: &App, platform: &Platform) -> bool {
        let body = self.body.get(app);
        let platform_body = platform.body.get(app);
        let character_bottom = body.position(app).y - body.size().y / 2.;
        let platform_top = platform_body.position(app).y + platform_body.size().y / 2.;
        let platform_bottom = platform_body.position(app).y - platform_body.size().y / 2.;
        character_bottom <= platform_top && character_bottom >= platform_bottom
    }

    fn force(&self, app: &App, is_touching_ground: bool, is_jump_pressed: bool) -> Vec2 {
        let gravity_force = Vec2::Y * GRAVITY_FACTOR * CHARACTER_MASS;
        let velocity = self.body.get(app).velocity(app);
        if is_touching_ground && is_jump_pressed && velocity.y.abs_diff_eq(&0., 0.001) {
            gravity_force + Vec2::Y * JUMP_FACTOR * CHARACTER_MASS
        } else {
            gravity_force
        }
    }
}
