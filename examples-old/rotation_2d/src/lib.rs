#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics::{Color, GraphicsModule, Mesh2D, SurfaceSize, WindowSettings};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, RelativeTransform2D, Transform2D};
use std::f32::consts::FRAC_PI_2;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default()
                .size(SurfaceSize::new(800, 600))
                .title("Modor - rotation 2D"),
        ))
        .with_entity(Object::build())
        .run(modor_graphics::runner);
}

#[derive(Component)]
struct Object;

#[systems]
impl Object {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(
                Transform2D::new()
                    .with_position(Vec2::new(0.15, 0.15))
                    .with_size(Vec2::new(0.25, 0.5)),
            )
            .with(Dynamics2D::new().with_angular_velocity(FRAC_PI_2))
            .with(Mesh2D::rectangle().with_color(Color::YELLOW))
            .with_child(Child::build())
    }
}

#[derive(Component)]
struct Child;

#[systems]
impl Child {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Transform2D::new().with_size(Vec2::new(0.1, 0.2)))
            .with(
                RelativeTransform2D::new()
                    .with_position(Vec2::ONE * 0.5)
                    .with_rotation(0.),
            )
            .with(Mesh2D::rectangle().with_color(Color::MAGENTA).with_z(1.))
    }
}
