#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, App, Built, EntityBuilder};
use modor_graphics::{
    Color, FrameRate, FrameRateLimit, GraphicsModule, Mesh, SurfaceSize, WindowSettings,
};
use modor_math::{Quat, Vec3};
use modor_physics::{DynamicBody, RelativeTransform, Transform};
use std::f32::consts::FRAC_PI_2;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default()
                .size(SurfaceSize::new(800, 600))
                .title("Modor - rotation 2D"),
        ))
        .with_entity(FrameRateLimit::build(FrameRate::VSync))
        .with_entity(Object::build())
        .run(modor_graphics::runner);
}

struct Object;

#[entity]
impl Object {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform::new()
                    .with_position(Vec3::xy(0.15, 0.15))
                    .with_size(Vec3::xyz(0.25, 0.5, 1.)),
            )
            .with(DynamicBody::new().with_angular_velocity(Quat::from_z(FRAC_PI_2)))
            .with(Mesh::rectangle().with_color(Color::YELLOW))
            .with_child(Child::build())
    }
}

struct Child;

#[entity]
impl Child {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform::new().with_size(Vec3::xyz(0.1, 0.2, 1.)))
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::ONE * 0.5)
                    .with_rotation(Quat::ZERO),
            )
            .with(DynamicBody::new().with_angular_velocity(Quat::from_z(FRAC_PI_2)))
            .with(Mesh::rectangle().with_color(Color::MAGENTA))
    }
}
