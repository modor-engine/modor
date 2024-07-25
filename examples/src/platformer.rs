use approx::AbsDiffEq;
use instant::Instant;
use modor::log::Level;
use modor::{App, RootNode, RootNodeHandle};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_input::{Inputs, Key};
use modor_graphics::{Color, Sprite2D};
use modor_physics::{Body2D, CollisionGroup, CollisionType, Impulse};
use std::time::Duration;

const PLATFORM_PERIOD: Duration = Duration::from_secs(4);
const CHARACTER_MASS: f32 = 1.;
const GRAVITY_FACTOR: f32 = -2.;
const JUMP_FACTOR: f32 = 50.;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root;

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        app.create::<Character>();
        app.create::<Platforms>();
        Self
    }
}

struct Platforms {
    platforms: Vec<Platform>,
}

impl RootNode for Platforms {
    fn on_create(app: &mut App) -> Self {
        Self {
            platforms: vec![
                // ground
                Platform::new(app, Vec2::new(0., -0.4), Vec2::new(1., 0.02), Vec2::ZERO),
                // wall
                Platform::new(app, Vec2::new(-0.5, 0.), Vec2::new(0.02, 0.82), Vec2::ZERO),
                // dynamic platforms
                Platform::new(
                    app,
                    Vec2::new(0., 0.2),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(0.15, 0.),
                ),
                Platform::new(
                    app,
                    Vec2::new(0., 0.05),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(-0.2, 0.),
                ),
                Platform::new(
                    app,
                    Vec2::new(0., -0.1),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(0.05, 0.),
                ),
                Platform::new(
                    app,
                    Vec2::new(0., -0.25),
                    Vec2::new(0.25, 0.02),
                    Vec2::new(-0.1, 0.),
                ),
            ],
        }
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
            .find(|platform| platform.body.glob().index() == body_index)
    }
}

struct CollisionGroups {
    platform: CollisionGroup,
    character: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(app: &mut App) -> Self {
        let platform = CollisionGroup::new(app);
        let character = CollisionGroup::new(app);
        let impulse = CollisionType::Impulse(Impulse::new(0., 0.));
        character.add_interaction(app, platform.glob(), impulse);
        Self {
            platform,
            character,
        }
    }

    fn update(&mut self, app: &mut App) {
        self.platform.update(app);
        self.character.update(app);
    }
}

struct Platform {
    body: Body2D,
    sprite: Sprite2D,
    next_reverse_instant: Instant,
}

impl Platform {
    fn new(app: &mut App, position: Vec2, size: Vec2, velocity: Vec2) -> Self {
        let collision_group = app.get_mut::<CollisionGroups>().platform.glob().to_ref();
        let body = Body2D::new(app)
            .with_position(position)
            .with_size(size)
            .with_velocity(velocity)
            .with_collision_group(Some(collision_group));
        let sprite = Sprite2D::new(app)
            .with_model(|m| m.body = Some(body.glob().to_ref()))
            .with_material(|m| m.color = Color::GREEN);
        Self {
            body,
            sprite,
            next_reverse_instant: Instant::now() + PLATFORM_PERIOD,
        }
    }

    fn update(&mut self, app: &mut App) {
        if Instant::now() >= self.next_reverse_instant {
            self.next_reverse_instant = Instant::now() + PLATFORM_PERIOD;
            self.body.velocity *= -1.;
        }
        self.body.update(app);
        self.sprite.update(app);
    }
}

struct Character {
    body: Body2D,
    sprite: Sprite2D,
    platforms: RootNodeHandle<Platforms>,
}

impl RootNode for Character {
    fn on_create(app: &mut App) -> Self {
        let collision_group = app.get_mut::<CollisionGroups>().character.glob().to_ref();
        let body = Body2D::new(app)
            .with_position(Vec2::new(0., 0.5))
            .with_size(Vec2::new(0.03, 0.1))
            .with_collision_group(Some(collision_group))
            .with_mass(CHARACTER_MASS)
            .with_force(Vec2::Y * GRAVITY_FACTOR * CHARACTER_MASS);
        let sprite = Sprite2D::new(app).with_model(|m| m.body = Some(body.glob().to_ref()));
        Self {
            body,
            sprite,
            platforms: app.handle(),
        }
    }

    fn update(&mut self, app: &mut App) {
        self.body.update(app); // force update to use latest information
        let keyboard = &app.get_mut::<Inputs>().keyboard;
        let x_movement = keyboard.axis(Key::ArrowLeft, Key::ArrowRight);
        let is_jump_pressed = keyboard[Key::ArrowUp].is_pressed();
        let touched_ground = self.touched_ground(app);
        let ground_velocity = touched_ground.map_or(0., |platform| platform.body.velocity.x);
        self.body.force = self.force(touched_ground.is_some(), is_jump_pressed);
        self.body.velocity.x = 0.5f32.mul_add(x_movement, ground_velocity);
        self.body.update(app);
        self.sprite.update(app);
    }
}

impl Character {
    fn touched_ground<'a>(&'a self, app: &'a App) -> Option<&Platform> {
        self.body
            .collisions()
            .iter()
            .filter_map(|collision| self.platforms.get(app).find(collision.other_index))
            .find(|platform| self.is_on_platform(platform))
    }

    fn is_on_platform(&self, platform: &Platform) -> bool {
        let character_bottom = self.body.position.y - self.body.size.y / 2.;
        let platform_top = platform.body.position.y + platform.body.size.y / 2.;
        let platform_bottom = platform.body.position.y - platform.body.size.y / 2.;
        character_bottom <= platform_top && character_bottom >= platform_bottom
    }

    fn force(&self, is_touching_ground: bool, is_jump_pressed: bool) -> Vec2 {
        let gravity_force = Vec2::Y * GRAVITY_FACTOR * CHARACTER_MASS;
        if is_touching_ground && is_jump_pressed && self.body.velocity.y.abs_diff_eq(&0., 0.001) {
            gravity_force + Vec2::Y * JUMP_FACTOR * CHARACTER_MASS
        } else {
            gravity_force
        }
    }
}
