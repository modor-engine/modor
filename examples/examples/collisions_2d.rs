#![allow(missing_docs)]

use modor::{systems, App, BuiltEntity, Component, EntityBuilder, EntityMut, Query, Single, World};
use modor_graphics::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_input::{InputModule, Mouse};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, PhysicsModule, RelativeTransform2D, Transform2D,
};
use modor_resources::ResKey;

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
const NOT_COLLIDING_CURSOR_MATERIAL: ResKey<Material> = ResKey::new("not-colliding-cursor");
const COLLIDING_CURSOR_MATERIAL: ResKey<Material> = ResKey::new("colliding-cursor");
const RECTANGLE_MATERIAL: ResKey<Material> = ResKey::new("rectangle");
const CIRCLE_MATERIAL: ResKey<Material> = ResKey::new("circle");
const CURSOR_COLLISION_POS_MATERIAL: ResKey<Material> = ResKey::new("cursor-collision-position");
const SHAPE_COLLISION_POS_MATERIAL: ResKey<Material> = ResKey::new("shape-collision-position");
const CURSOR_COLLISION_DIR_MATERIAL: ResKey<Material> = ResKey::new("cursor-collision-direction");
const SHAPE_COLLISION_DIR_MATERIAL: ResKey<Material> = ResKey::new("shape-collision-direction");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(InputModule::build())
        .with_entity(modor_graphics::module())
        .with_entity(window())
        .with_entity(materials())
        .with_entity(cursor())
        .with_entity(rectangle())
        .with_entity(circle())
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity {
    let target_key = ResKey::unique("window");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Window::default())
        .with(|w| w.is_cursor_shown = false)
        .component(Camera2D::new(CAMERA, target_key))
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Material::new(NOT_COLLIDING_CURSOR_MATERIAL))
        .with(|m| m.color = Color::GREEN)
        .child_component(Material::new(NOT_COLLIDING_CURSOR_MATERIAL))
        .with(|m| m.color = Color::GREEN)
        .child_component(Material::new(COLLIDING_CURSOR_MATERIAL))
        .with(|m| m.color = Color::RED)
        .child_component(Material::new(RECTANGLE_MATERIAL))
        .with(|m| m.color = Color::BLUE)
        .child_component(Material::ellipse(CIRCLE_MATERIAL))
        .with(|m| m.color = Color::BLUE)
        .child_component(Material::ellipse(CURSOR_COLLISION_POS_MATERIAL))
        .with(|m| m.color = Color::YELLOW)
        .child_component(Material::ellipse(SHAPE_COLLISION_POS_MATERIAL))
        .with(|m| m.color = Color::DARK_GRAY)
        .child_component(Material::new(CURSOR_COLLISION_DIR_MATERIAL))
        .with(|m| m.color = Color::YELLOW)
        .child_component(Material::new(SHAPE_COLLISION_DIR_MATERIAL))
        .with(|m| m.color = Color::DARK_GRAY)
}

fn cursor() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(0.05, 0.1))
        .component(Collider2D::rectangle(CollisionGroup::Cursor))
        .component(Model::rectangle(NOT_COLLIDING_CURSOR_MATERIAL, CAMERA))
        .component(ZIndex2D::from(1))
        .component(Cursor)
}

fn rectangle() -> impl BuiltEntity {
    let position = Vec2::X * 0.25;
    let size = Vec2::new(0.2, 0.3);
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = position)
        .with(|t| *t.size = size)
        .component(Collider2D::rectangle(CollisionGroup::Shape))
        .component(Model::rectangle(RECTANGLE_MATERIAL, CAMERA))
        .component(Shape)
}

fn circle() -> impl BuiltEntity {
    let position = -Vec2::X * 0.25;
    let size = Vec2::ONE * 0.4;
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = position)
        .with(|t| *t.size = size)
        .component(Collider2D::circle(CollisionGroup::Shape))
        .component(Model::rectangle(CIRCLE_MATERIAL, CAMERA))
        .component(Shape)
}

fn collision_mark(position: Vec2, normal: Vec2, is_cursor: bool) -> impl BuiltEntity {
    let material_key = if is_cursor {
        CURSOR_COLLISION_POS_MATERIAL
    } else {
        SHAPE_COLLISION_POS_MATERIAL
    };
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = position)
        .with(|t| *t.size = Vec2::ONE * 0.02)
        .with(|t| *t.rotation = Vec2::X.rotation(normal))
        .component(Model::rectangle(material_key, CAMERA))
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
    let material_key = if is_cursor {
        CURSOR_COLLISION_DIR_MATERIAL
    } else {
        SHAPE_COLLISION_DIR_MATERIAL
    };
    EntityBuilder::new()
        .component(Transform2D::new())
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::X * 0.5))
        .with(|t| t.size = Some(Vec2::ONE))
        .with(|t| t.rotation = Some(0.))
        .component(Model::rectangle(material_key, CAMERA))
        .component(ZIndex2D::from(2))
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
            NOT_COLLIDING_CURSOR_MATERIAL
        } else {
            COLLIDING_CURSOR_MATERIAL
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
    fn remove(mut entity: EntityMut<'_>) {
        entity.delete();
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
