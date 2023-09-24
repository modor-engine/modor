#![allow(missing_docs)]

use modor::{
    systems, App, BuiltEntity, Component, EntityBuilder, Single, SingleRef, TemporaryComponent,
    World,
};
use modor_graphics::{
    model_2d, window_target, Camera2D, Color, Material, Model2DMaterial, Window, ZIndex2D,
    WINDOW_CAMERA_2D,
};
use modor_input::Mouse;
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, PhysicsModule, RelativeTransform2D, Transform2D,
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics::module())
        .with_entity(window_target().updated(|w: &mut Window| w.is_cursor_shown = false))
        .with_entity(cursor())
        .with_entity(rectangle())
        .with_entity(circle())
        .run(modor_graphics::runner);
}

fn cursor() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.05, 0.1))
        .updated(|m: &mut Material| m.color = Color::GREEN)
        .component(Collider2D::rectangle(CollisionGroup::Cursor))
        .component(ZIndex2D::from(1))
        .component(Cursor)
}

fn rectangle() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| *t.position = Vec2::X * 0.25)
        .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.2, 0.3))
        .updated(|m: &mut Material| m.color = Color::CYAN)
        .component(Collider2D::rectangle(CollisionGroup::Shape))
        .component(Shape)
}

fn circle() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| *t.position = -Vec2::X * 0.25)
        .updated(|t: &mut Transform2D| *t.size = Vec2::ONE * 0.4)
        .updated(|m: &mut Material| m.color = Color::CYAN)
        .component(Collider2D::circle(CollisionGroup::Shape))
        .component(Shape)
}

fn collision_mark(position: Vec2, normal: Vec2, is_cursor: bool) -> impl BuiltEntity {
    let color = if is_cursor {
        Color::YELLOW
    } else {
        Color::DARK_GRAY
    };
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| *t.position = position)
        .updated(|t: &mut Transform2D| *t.size = Vec2::ONE * 0.02)
        .updated(|t: &mut Transform2D| *t.rotation = Vec2::X.rotation(normal))
        .updated(|m: &mut Material| m.color = color)
        .component(ZIndex2D::from(2))
        .component(AutoRemoved)
        .child_entity(collision_normal(is_cursor))
}

fn collision_normal(is_cursor: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(0.05, 0.005))
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::ZERO))
        .with(|t| t.rotation = Some(0.))
        .child_entity(collision_normal_rectangle(is_cursor))
}

fn collision_normal_rectangle(is_cursor: bool) -> impl BuiltEntity {
    let color = if is_cursor {
        Color::YELLOW
    } else {
        Color::DARK_GRAY
    };
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|m: &mut Material| m.color = color)
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::X * 0.5))
        .with(|t| t.size = Some(Vec2::ONE))
        .with(|t| t.rotation = Some(0.))
        .component(ZIndex2D::from(2))
}

#[derive(Component)]
struct Cursor;

#[systems]
impl Cursor {
    #[run]
    fn update_position(
        transform: &mut Transform2D,
        mouse: SingleRef<'_, '_, Mouse>,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
    ) {
        let (window, camera) = window_camera.get();
        *transform.position = camera.world_position(window.size(), mouse.get().position);
    }

    #[run]
    fn update_material(material: &mut Material, collider: &Collider2D, mut world: World<'_>) {
        material.color = if collider.collisions().is_empty() {
            Color::GREEN
        } else {
            Color::RED
        };
        for collision in collider.collisions() {
            world.create_root_entity(collision_mark(collision.position, collision.normal, true));
        }
    }
}

#[derive(Component)]
struct Shape;

#[systems]
impl Shape {
    #[run]
    fn create_collision_marks(collider: &Collider2D, mut world: World<'_>) {
        for collision in collider.collisions() {
            world.create_root_entity(collision_mark(collision.position, collision.normal, false));
        }
    }
}

#[derive(Component, TemporaryComponent)]
struct AutoRemoved;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum CollisionGroup {
    Cursor,
    Shape,
}

impl CollisionGroupRef for CollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Cursor, Self::Shape) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}
