#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{systems, App, BuiltEntity, Component, Entity, EntityBuilder, Single, World};
use modor_graphics::{Color, GraphicsModule, Mesh2D, WindowSettings};
use modor_input::{Key, Keyboard};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, DeltaTime, Dynamics2D, PhysicsModule,
    RelativeTransform2D, Transform2D,
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default().title("Modor - collisions"),
        ))
        .with_entity(PhysicsModule::build())
        .with_entity(Character::build(Vec2::ZERO, Vec2::new(0.05, 0.1)))
        .with_entity(Rectangle::build(Vec2::X * 0.25, Vec2::new(0.2, 0.3)))
        .with_entity(Circle::build(Vec2::X * -0.25, 0.2))
        .run(modor_graphics::runner);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum CollisionGroup {
    Character,
    Object,
}

impl CollisionGroupRef for CollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Character, Self::Object) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}

#[derive(Component)]
struct Character;

#[systems]
impl Character {
    fn build(position: Vec2, size: Vec2) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Transform2D::new().with_position(position).with_size(size))
            .with(Dynamics2D::new())
            .with(Mesh2D::rectangle().with_color(Color::GREEN).with_z(1.))
            .with(Collider2D::rectangle(CollisionGroup::Character))
    }

    #[run]
    fn update_dynamics(dynamics: &mut Dynamics2D, keyboard: Single<'_, Keyboard>) {
        let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
        let rotation = keyboard.axis(Key::Key1, Key::Key2);
        *dynamics.velocity = direction * 0.5;
        *dynamics.angular_velocity = rotation * 2.;
    }

    #[run]
    fn update_transform(
        transform: &mut Transform2D,
        keyboard: Single<'_, Keyboard>,
        delta: Single<'_, DeltaTime>,
    ) {
        let scale = keyboard.axis(Key::Key3, Key::Key4);
        *transform.size += Vec2::ONE * scale * delta.get().as_secs_f32() * 0.2;
    }

    #[run]
    fn update_color(mesh: &mut Mesh2D, collider: &Collider2D, mut world: World<'_>) {
        mesh.color = if collider.collisions().is_empty() {
            Color::GREEN
        } else {
            Color::RED
        };
        for collision in collider.collisions() {
            world.create_root_entity(CollisionPosition::build(
                collision.position,
                collision.normal,
                Color::DARK_GRAY,
            ));
        }
    }
}

#[derive(Component)]
struct Object;

#[systems]
impl Object {
    fn build(position: Vec2, size: Vec2) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Transform2D::new().with_position(position).with_size(size))
    }

    #[run]
    fn update_color(mesh: &mut Mesh2D, collider: &Collider2D, mut world: World<'_>) {
        mesh.color = if collider.collisions().is_empty() {
            Color::BLUE
        } else {
            Color::CYAN
        };
        for collision in collider.collisions() {
            world.create_root_entity(CollisionPosition::build(
                collision.position,
                collision.normal,
                Color::YELLOW,
            ));
        }
    }
}

#[derive(Component)]
struct Rectangle;

#[systems]
impl Rectangle {
    fn build(position: Vec2, size: Vec2) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with_inherited(Object::build(position, size))
            .with(Mesh2D::rectangle().with_color(Color::BLUE))
            .with(Collider2D::rectangle(CollisionGroup::Object))
    }
}

#[derive(Component)]
struct Circle;

#[systems]
impl Circle {
    fn build(position: Vec2, radius: f32) -> impl BuiltEntity {
        let size = Vec2::ONE * radius * 2.;
        EntityBuilder::new()
            .with(Self)
            .with_inherited(Object::build(position, size))
            .with(Mesh2D::ellipse().with_color(Color::BLUE))
            .with(Collider2D::circle(CollisionGroup::Object))
    }
}

#[derive(Component)]
struct CollisionPosition;

#[systems]
impl CollisionPosition {
    fn build(position: Vec2, normal: Vec2, color: Color) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::ONE * 0.02)
                    .with_rotation(Vec2::X.rotation(normal)),
            )
            .with(Mesh2D::ellipse().with_color(color).with_z(2.))
            .with_child(CollisionNormal::build(color))
    }

    #[run]
    fn keep_only_one_frame(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

#[derive(Component)]
struct CollisionNormal;

#[systems]
impl CollisionNormal {
    fn build(color: Color) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Transform2D::new().with_size(Vec2::new(0.05, 0.005)))
            .with(
                RelativeTransform2D::new()
                    .with_position(Vec2::ZERO)
                    .with_rotation(0.),
            )
            .with_child(CollisionNormalDirection::build(color))
    }
}

#[derive(Component)]
struct CollisionNormalDirection;

#[systems]
impl CollisionNormalDirection {
    fn build(color: Color) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Transform2D::new())
            .with(
                RelativeTransform2D::new()
                    .with_position(Vec2::X * 0.5)
                    .with_size(Vec2::ONE)
                    .with_rotation(0.),
            )
            .with(Mesh2D::rectangle().with_color(color).with_z(2.))
    }
}
