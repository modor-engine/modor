use modor::{systems, App, BuiltEntity, Component, Single, SingleRef, World};
use modor_graphics::{
    model_2d, window_target, Camera2D, Color, Material, Model2DMaterial, Window, WINDOW_CAMERA_2D,
};
use modor_input::{Fingers, Mouse, MouseButton};
use modor_math::Vec2;
use modor_physics::{Collider2D, CollisionGroup, CollisionType, Dynamics2D, Impulse, Transform2D};
use modor_resources::ResKey;
use rand::Rng;

const WALL_GROUP: ResKey<CollisionGroup> = ResKey::new("wall");
const OBJECT_GROUP: ResKey<CollisionGroup> = ResKey::new("object");

const GRAVITY: f32 = 2.;
const CANNON_JOIN_POSITION: Vec2 = Vec2::new(0., 0.6);
const CANNON_LENGTH: f32 = 0.3;
const OBJECT_MASS: f32 = 10.;
const OBJECT_RADIUS: f32 = 0.04;
const OBJECT_INITIAL_SPEED: f32 = 1.;

const RECTANGLE_INERTIA_FACTOR: f32 = 1. / 3.;
const CIRCLE_INERTIA_FACTOR: f32 = 1. / 4.;

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window_target())
        .with_entity(CollisionGroup::new(WALL_GROUP, wall_collision_type))
        .with_entity(CollisionGroup::new(OBJECT_GROUP, object_collision_type))
        .with_entity(horizontal_wall(-0.5))
        .with_entity(vertical_wall(-0.5))
        .with_entity(vertical_wall(0.5))
        .with_entity(cannon())
        .run(modor_graphics::runner);
}

fn wall_collision_type(_group_key: ResKey<CollisionGroup>) -> CollisionType {
    CollisionType::None
}

fn object_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == WALL_GROUP || group_key == OBJECT_GROUP {
        CollisionType::Impulse(Impulse::new(0.1, 0.8))
    } else {
        CollisionType::None
    }
}

fn vertical_wall(x: f32) -> impl BuiltEntity {
    wall()
        .updated(|t: &mut Transform2D| t.position = Vec2::new(x, 0.))
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.03, 1.))
}

fn horizontal_wall(y: f32) -> impl BuiltEntity {
    wall()
        .updated(|t: &mut Transform2D| t.position = Vec2::new(0., y))
        .updated(|t: &mut Transform2D| t.size = Vec2::new(1., 0.03))
}

fn wall() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .component(Collider2D::rectangle(WALL_GROUP))
}

fn cannon() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.05, CANNON_LENGTH))
        .component(Cannon)
}

fn box_(position: Vec2, velocity: Vec2) -> impl BuiltEntity {
    object(
        position,
        velocity,
        RECTANGLE_INERTIA_FACTOR,
        Model2DMaterial::Rectangle,
        Collider2D::rectangle(OBJECT_GROUP),
    )
}

fn ball(position: Vec2, velocity: Vec2) -> impl BuiltEntity {
    object(
        position,
        velocity,
        CIRCLE_INERTIA_FACTOR,
        Model2DMaterial::Ellipse,
        Collider2D::circle(OBJECT_GROUP),
    )
}

fn object(
    position: Vec2,
    velocity: Vec2,
    inertia_factor: f32,
    material: Model2DMaterial,
    collider: Collider2D,
) -> impl BuiltEntity {
    let mut rng = rand::thread_rng();
    let color = Color::rgb(
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
        rng.gen_range(0.0..1.0),
    );
    model_2d(WINDOW_CAMERA_2D, material)
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * OBJECT_RADIUS * 2.)
        .updated(|t: &mut Material| t.color = color)
        .component(Dynamics2D::new())
        .with(|d| d.velocity = velocity)
        .with(|d| d.force = -Vec2::Y * GRAVITY * OBJECT_MASS)
        .with(|d| d.mass = OBJECT_MASS)
        .with(|d| d.angular_inertia = OBJECT_MASS * OBJECT_RADIUS.powi(2) / inertia_factor)
        .component(collider)
}

#[derive(Component)]
struct Cannon;

#[systems]
impl Cannon {
    #[run]
    fn update(
        transform: &mut Transform2D,
        mouse: SingleRef<'_, '_, Mouse>,
        fingers: SingleRef<'_, '_, Fingers>,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
        mut world: World<'_>,
    ) {
        let mouse = mouse.get();
        let fingers = fingers.get();
        let mouse_position = Self::cursor_position(mouse, fingers, window_camera);
        transform.rotation = Vec2::Y.rotation(mouse_position - CANNON_JOIN_POSITION);
        transform.position = Self::position(transform);
        if Self::is_cursor_clicked(mouse, fingers) {
            let position = Self::object_initial_position(transform);
            let velocity = Vec2::Y.with_rotation(transform.rotation) * OBJECT_INITIAL_SPEED;
            if mouse[MouseButton::Right].is_just_released() {
                world.create_root_entity(ball(position, velocity));
            } else {
                world.create_root_entity(box_(position, velocity));
            }
        }
    }

    fn position(transform: &mut Transform2D) -> Vec2 {
        CANNON_JOIN_POSITION + (Vec2::Y * CANNON_LENGTH / 2.).with_rotation(transform.rotation)
    }

    fn object_initial_position(transform: &mut Transform2D) -> Vec2 {
        CANNON_JOIN_POSITION
            + (Vec2::Y * (CANNON_LENGTH + OBJECT_RADIUS / 2.)).with_rotation(transform.rotation)
    }

    fn cursor_position(
        mouse: &Mouse,
        fingers: &Fingers,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
    ) -> Vec2 {
        let (window, camera) = window_camera.get();
        let surface_position = fingers
            .iter()
            .next()
            .map_or(mouse.position, |finger_id| fingers[finger_id].position);
        camera.world_position(window.size(), surface_position)
    }

    fn is_cursor_clicked(mouse: &Mouse, fingers: &Fingers) -> bool {
        fingers.iter().next().map_or_else(
            || {
                mouse[MouseButton::Left].is_just_released()
                    || mouse[MouseButton::Right].is_just_released()
            },
            |finger_id| fingers[finger_id].state.is_just_released(),
        )
    }
}
