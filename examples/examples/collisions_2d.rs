#![allow(missing_docs)]

use modor::{systems, App, BuiltEntity, Component, Entity, EntityBuilder, Query, Single, World};
use modor_graphics_new2::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_input::{InputModule, Mouse};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, PhysicsModule, RelativeTransform2D, Transform2D,
};
use modor_resources::IntoResourceKey;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(InputModule::build())
        .with_entity(modor_graphics_new2::module())
        .with_entity(window())
        .with_entity(Material::new(MaterialKey::NotCollidingCursor).with_color(Color::GREEN))
        .with_entity(Material::new(MaterialKey::CollidingCursor).with_color(Color::RED))
        .with_entity(Material::new(MaterialKey::Rectangle).with_color(Color::BLUE))
        .with_entity(Material::ellipse(MaterialKey::Circle).with_color(Color::BLUE))
        .with_entity(Material::ellipse(MaterialKey::CursorCollisionPos).with_color(Color::YELLOW))
        .with_entity(Material::ellipse(MaterialKey::ShapeCollisionPos).with_color(Color::DARK_GRAY))
        .with_entity(Material::new(MaterialKey::CursorCollisionDir).with_color(Color::YELLOW))
        .with_entity(Material::new(MaterialKey::ShapeCollisionDir).with_color(Color::DARK_GRAY))
        .with_entity(cursor())
        .with_entity(rectangle())
        .with_entity(circle())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default().with_cursor_shown(false))
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn cursor() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.05, 0.1)))
        .with(Collider2D::rectangle(CollisionGroup::Cursor))
        .with(Model::rectangle(MaterialKey::NotCollidingCursor).with_camera_key(CameraKey))
        .with(ZIndex2D::from(1))
        .with(Cursor)
}

fn rectangle() -> impl BuiltEntity {
    let position = Vec2::X * 0.25;
    let size = Vec2::new(0.2, 0.3);
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position).with_size(size))
        .with(Collider2D::rectangle(CollisionGroup::Shape))
        .with(Model::rectangle(MaterialKey::Rectangle).with_camera_key(CameraKey))
        .with(Shape)
}

fn circle() -> impl BuiltEntity {
    let position = -Vec2::X * 0.25;
    let size = Vec2::ONE * 0.4;
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position).with_size(size))
        .with(Collider2D::circle(CollisionGroup::Shape))
        .with(Model::rectangle(MaterialKey::Circle).with_camera_key(CameraKey))
        .with(Shape)
}

fn collision_mark(position: Vec2, normal: Vec2, is_cursor: bool) -> impl BuiltEntity {
    let size = Vec2::ONE * 0.02;
    let material_key = if is_cursor {
        MaterialKey::CursorCollisionPos
    } else {
        MaterialKey::ShapeCollisionPos
    };
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(size)
                .with_rotation(Vec2::X.rotation(normal)),
        )
        .with(Model::rectangle(material_key).with_camera_key(CameraKey))
        .with(ZIndex2D::from(2))
        .with(AutoRemoved)
        .with_child(collision_normal(is_cursor))
}

fn collision_normal(is_cursor: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.05, 0.005)))
        .with(
            RelativeTransform2D::new()
                .with_position(Vec2::ZERO)
                .with_rotation(0.),
        )
        .with_child(collision_normal_rectangle(is_cursor))
}

fn collision_normal_rectangle(is_cursor: bool) -> impl BuiltEntity {
    let material_key = if is_cursor {
        MaterialKey::CursorCollisionDir
    } else {
        MaterialKey::ShapeCollisionDir
    };
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(
            RelativeTransform2D::new()
                .with_position(Vec2::X * 0.5)
                .with_size(Vec2::ONE)
                .with_rotation(0.),
        )
        .with(Model::rectangle(material_key).with_camera_key(CameraKey))
        .with(ZIndex2D::from(2))
}

#[derive(Component)]
struct Cursor;

#[systems]
impl Cursor {
    #[run]
    fn update_position(
        transform: &mut Transform2D,
        mouse: Single<'_, Mouse>,
        window: Single<'_, Window>,
        cameras: Query<'_, &Camera2D>,
    ) {
        let Some(camera) = cameras.iter().next() else { return; };
        *transform.position = camera.world_position(window.size(), mouse.position());
    }

    #[run]
    fn update_material(model: &mut Model, collider: &Collider2D, mut world: World<'_>) {
        model.material_key = if collider.collisions().is_empty() {
            MaterialKey::NotCollidingCursor.into_key()
        } else {
            MaterialKey::CollidingCursor.into_key()
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

#[derive(Component)]
struct AutoRemoved;

#[systems]
impl AutoRemoved {
    #[run]
    fn remove(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    NotCollidingCursor,
    CollidingCursor,
    Rectangle,
    Circle,
    CursorCollisionPos,
    ShapeCollisionPos,
    CursorCollisionDir,
    ShapeCollisionDir,
}
