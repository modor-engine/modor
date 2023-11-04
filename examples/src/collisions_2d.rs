use modor::{systems, App, BuiltEntity, Component, Single, SingleRef, TemporaryComponent, World};
use modor_graphics::{
    model_2d, window_target, Camera2D, Color, Material, Model2DMaterial, Window, ZIndex2D,
    WINDOW_CAMERA_2D,
};
use modor_input::Mouse;
use modor_math::Vec2;
use modor_physics::{Collider2D, Collision2D, CollisionGroup, CollisionType, Transform2D};
use modor_resources::ResKey;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

const CURSOR_GROUP: ResKey<CollisionGroup> = ResKey::new("cursor");
const SHAPE_GROUP: ResKey<CollisionGroup> = ResKey::new("shape");

pub fn main() {
    App::new()
        .with_entity(modor_physics::module())
        .with_entity(modor_graphics::module())
        .with_entity(CollisionGroup::new(CURSOR_GROUP, cursor_collision_type))
        .with_entity(CollisionGroup::new(SHAPE_GROUP, shape_collision_type))
        .with_entity(window_target().updated(|w: &mut Window| w.is_cursor_shown = false))
        .with_entity(cursor())
        .with_entity(rectangle())
        .with_entity(circle())
        .run(modor_graphics::runner);
}

fn cursor_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == SHAPE_GROUP {
        CollisionType::Sensor
    } else {
        CollisionType::None
    }
}

fn shape_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == CURSOR_GROUP {
        CollisionType::Sensor
    } else {
        CollisionType::None
    }
}

fn cursor() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.05, 0.1))
        .updated(|t: &mut Transform2D| t.rotation = FRAC_PI_8)
        .updated(|m: &mut Material| m.color = Color::GREEN)
        .component(Collider2D::rectangle(CURSOR_GROUP))
        .component(ZIndex2D::from(1))
        .component(Cursor)
}

fn rectangle() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.position = Vec2::X * 0.25)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.2, 0.3))
        .updated(|m: &mut Material| m.color = Color::CYAN)
        .component(Collider2D::rectangle(SHAPE_GROUP))
        .component(Shape)
}

fn circle() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.position = -Vec2::X * 0.25)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.4)
        .updated(|m: &mut Material| m.color = Color::CYAN)
        .component(Collider2D::circle(SHAPE_GROUP))
        .component(Shape)
}

fn collision_mark(collision: &Collision2D, is_cursor: bool) -> impl BuiltEntity {
    let color = if is_cursor {
        Color::YELLOW
    } else {
        Color::DARK_GRAY
    };
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.position = collision.position)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.02)
        .updated(|m: &mut Material| m.color = color)
        .component(ZIndex2D::from(if is_cursor { 2 } else { 3 }))
        .component(AutoRemoved)
        .child_entity(collision_penetration(collision, is_cursor))
}

fn collision_penetration(collision: &Collision2D, is_cursor: bool) -> impl BuiltEntity {
    let color = if is_cursor {
        Color::YELLOW
    } else {
        Color::DARK_GRAY
    };
    let lateral_offset = collision
        .penetration
        .with_rotation(FRAC_PI_2)
        .with_magnitude(0.0025)
        .unwrap_or_default();
    let position = collision.position - collision.penetration / 2. + lateral_offset;
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.005, collision.penetration.magnitude()))
        .updated(|t: &mut Transform2D| t.rotation = Vec2::Y.rotation(-collision.penetration))
        .updated(|m: &mut Material| m.color = color)
        .component(ZIndex2D::from(if is_cursor { 2 } else { 3 }))
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
        transform.position = camera.world_position(window.size(), mouse.get().position);
    }

    #[run_after(component(Collider2D))]
    fn update_material(material: &mut Material, collider: &Collider2D, mut world: World<'_>) {
        material.color = if collider.collisions().is_empty() {
            Color::GREEN
        } else {
            Color::RED
        };
        for collision in collider.collisions() {
            world.create_root_entity(collision_mark(collision, true));
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
            world.create_root_entity(collision_mark(collision, false));
        }
    }
}

#[derive(Component, TemporaryComponent)]
struct AutoRemoved;
