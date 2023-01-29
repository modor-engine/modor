// TODO: remove this file
use modor::{entity, App, Built, EntityBuilder};
use modor_graphics_new::{Color, GraphicsModule, Mesh2D, WindowTitle};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};

fn main() {
    test_window();
}

struct Rectangle;

#[entity]
impl Rectangle {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform2D::new().with_size(Vec2::new(0.5, 0.3)))
            .with(Dynamics2D::new().with_angular_velocity(0.2))
            .with(Mesh2D::ellipse().with_color(Color::GREEN))
    }
}

pub fn test_window() {
    App::new()
        .with_entity(GraphicsModule::build())
        .with_entity(WindowTitle::build("Example"))
        .with_entity(Rectangle::build())
        .run(modor_graphics_new::runner);
}

pub fn test_capture() {
    App::new()
        .with_entity(GraphicsModule::build_windowless())
        .update();
}
