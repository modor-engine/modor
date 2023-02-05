// TODO: remove this file
use modor::{entity, App, Built, EntityBuilder};
use modor_graphics_new::{Camera2D, CameraRef, Color, GraphicsModule, Mesh2D, WindowTitle};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};

fn main() {
    test_window();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CustomCamera;

impl CameraRef for CustomCamera {}

#[entity]
impl CustomCamera {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Camera2D::build(Self))
            .with(
                Transform2D::new()
                    .with_position(0.4 * Vec2::ONE)
                    .with_size(2. * Vec2::ONE),
            )
    }
}

struct DefaultObject;

#[entity]
impl DefaultObject {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform2D::new().with_size(Vec2::new(0.5, 0.3)))
            .with(Dynamics2D::new().with_angular_velocity(0.2))
            .with(Mesh2D::ellipse().with_color(Color::GREEN).with_z(-0.49999))
    }
}

struct CustomObject;

#[entity]
impl CustomObject {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform2D::new().with_size(Vec2::new(0.4, 0.4)))
            .with(Dynamics2D::new().with_angular_velocity(-0.5))
            .with(
                Mesh2D::rectangle()
                    .with_color(Color::BLUE)
                    .with_camera(CustomCamera)
                    .with_z(0.5),
            )
    }
}

pub fn test_window() {
    App::new()
        .with_entity(GraphicsModule::build())
        .with_entity(WindowTitle::build("Example"))
        .with_entity(CustomCamera::build())
        .with_entity(DefaultObject::build())
        .with_entity(CustomObject::build())
        .run(modor_graphics_new::runner);
}

pub fn test_capture() {
    App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(CustomCamera::build())
        .with_entity(DefaultObject::build())
        .with_entity(CustomObject::build())
        .update();
}
