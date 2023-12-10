use approx::AbsDiffEq;
use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, Query, SingleRef, SingletonComponent};
use modor_graphics::{instance_2d, window_target, Color, Material, WINDOW_CAMERA_2D};
use modor_input::{Key, Keyboard};
use modor_math::Vec2;
use modor_physics::{Collider2D, CollisionGroup, CollisionType, Dynamics2D, Impulse, Transform2D};
use modor_resources::ResKey;
use std::time::Duration;

const PLATFORM_GROUP: ResKey<CollisionGroup> = ResKey::new("platform-bottom");
const CHARACTER_GROUP: ResKey<CollisionGroup> = ResKey::new("character");

const PLATFORM_PERIOD: Duration = Duration::from_secs(4);
const CHARACTER_MASS: f32 = 1.;
const GRAVITY_FACTOR: f32 = -2.;
const JUMP_FACTOR: f32 = 50.;

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window_target())
        .with_entity(CollisionGroup::new(PLATFORM_GROUP, |_| CollisionType::None))
        .with_entity(CollisionGroup::new(
            CHARACTER_GROUP,
            character_collision_type,
        ))
        .with_entity(platform(
            // lowest ground
            Vec2::new(0., -0.4),
            Vec2::new(1., 0.02),
            Vec2::ZERO,
        ))
        .with_entity(platform(
            // wall
            Vec2::new(-0.5, 0.),
            Vec2::new(0.02, 0.82),
            Vec2::ZERO,
        ))
        .with_entity(platform(
            // dynamic platform
            Vec2::new(0., 0.2),
            Vec2::new(0.25, 0.02),
            Vec2::new(0.15, 0.),
        ))
        .with_entity(platform(
            // dynamic platform
            Vec2::new(0., 0.05),
            Vec2::new(0.25, 0.02),
            Vec2::new(-0.2, 0.),
        ))
        .with_entity(platform(
            // dynamic platform
            Vec2::new(0., -0.1),
            Vec2::new(0.25, 0.02),
            Vec2::new(0.05, 0.),
        ))
        .with_entity(platform(
            // dynamic platform
            Vec2::new(0., -0.25),
            Vec2::new(0.25, 0.02),
            Vec2::new(-0.1, 0.),
        ))
        .with_entity(character())
        .run(modor_graphics::runner);
}

fn character_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == PLATFORM_GROUP {
        CollisionType::Impulse(Impulse::new(0., 0.))
    } else {
        CollisionType::None
    }
}

fn platform(position: Vec2, size: Vec2, velocity: Vec2) -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = size)
        .updated(|t: &mut Material| t.color = Color::GREEN)
        .component(Dynamics2D::new())
        .with(|d| d.velocity = velocity)
        .component(Collider2D::rectangle(PLATFORM_GROUP))
        .component(PlatformMovement::new())
}

fn character() -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|t: &mut Transform2D| t.position = Vec2::new(0., 0.5))
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.03, 0.1))
        .component(Dynamics2D::new())
        .with(|d| d.mass = CHARACTER_MASS)
        .with(|d| d.force = Vec2::Y * GRAVITY_FACTOR * CHARACTER_MASS)
        .component(Collider2D::rectangle(CHARACTER_GROUP))
        .component(CharacterController {
            jump_key: Key::ArrowUp,
            left_key: Key::ArrowLeft,
            right_key: Key::ArrowRight,
            next_velocity_x: 0.,
            next_force: Vec2::ZERO,
        })
}

#[derive(Component)]
struct PlatformMovement {
    next_reverse_instant: Instant,
}

#[systems]
impl PlatformMovement {
    fn new() -> Self {
        Self {
            next_reverse_instant: Instant::now() + PLATFORM_PERIOD,
        }
    }

    #[run]
    fn update(&mut self, dynamics: &mut Dynamics2D) {
        if Instant::now() >= self.next_reverse_instant {
            self.next_reverse_instant = Instant::now() + PLATFORM_PERIOD;
            dynamics.velocity *= -1.;
        }
    }
}

#[derive(SingletonComponent)]
struct CharacterController {
    jump_key: Key,
    left_key: Key,
    right_key: Key,
    next_velocity_x: f32,
    next_force: Vec2,
}

#[systems]
impl CharacterController {
    #[run]
    fn update(
        &mut self,
        transform: &Transform2D,
        dynamics: &Dynamics2D,
        collider: &Collider2D,
        keyboard: SingleRef<'_, '_, Keyboard>,
        objects: Query<'_, (&Transform2D, &Dynamics2D)>,
    ) {
        let keyboard = keyboard.get();
        let jump_pressed = keyboard[self.jump_key].is_pressed();
        let touched_ground = Self::touched_platform(transform, collider, &objects);
        let is_on_platform =
            touched_ground.is_some() && dynamics.velocity.y.abs_diff_eq(&0., 0.001);
        self.next_velocity_x = 0.5f32.mul_add(
            keyboard.axis(self.left_key, self.right_key),
            touched_ground.map_or(0., |(_, dynamics)| dynamics.velocity.x),
        );
        self.next_force = Self::force(is_on_platform, jump_pressed);
    }

    #[run_after_previous]
    fn update_dynamics(&self, dynamics: &mut Dynamics2D) {
        dynamics.velocity.x = self.next_velocity_x;
        dynamics.force = self.next_force;
    }

    fn force(is_on_ground: bool, jump_pressed: bool) -> Vec2 {
        let gravity_force = Vec2::Y * GRAVITY_FACTOR * CHARACTER_MASS;
        if is_on_ground && jump_pressed {
            gravity_force + Vec2::Y * JUMP_FACTOR * CHARACTER_MASS
        } else {
            gravity_force
        }
    }

    fn touched_platform<'a>(
        transform: &Transform2D,
        collider: &'a Collider2D,
        objects: &'a Query<'_, (&Transform2D, &Dynamics2D)>,
    ) -> Option<(&'a Transform2D, &'a Dynamics2D)> {
        collider
            .collided_with(objects, PLATFORM_GROUP)
            .filter(|(_, (other_transform, _))| Self::is_on_platform(transform, other_transform))
            .map(|(_, (transform, dynamics))| (transform, dynamics))
            .next()
    }

    fn is_on_platform(character_transform: &Transform2D, platform_transform: &Transform2D) -> bool {
        let character_bottom = character_transform.position.y - character_transform.size.y / 2.;
        let platform_top = platform_transform.position.y + platform_transform.size.y / 2.;
        let platform_bottom = platform_transform.position.y - platform_transform.size.y / 2.;
        character_bottom <= platform_top && character_bottom >= platform_bottom
    }
}
