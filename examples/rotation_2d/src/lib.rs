#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, App, Built, EntityBuilder};
use modor_graphics::{
    Color, FrameRate, FrameRateLimit, GraphicsModule, ShapeColor, SurfaceSize, WindowSettings,
};
use modor_math::{Quat, Vec3};
use modor_physics::{
    AngularVelocity, Position, RelativeAngularVelocity, RelativePosition, RelativeRotation,
    Rotation, Size,
};
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
            .with(Position::from(Vec3::xy(0.15, 0.15)))
            .with(Size::from(Vec3::xyz(0.25, 0.5, 1.)))
            .with(ShapeColor::from(Color::rgb(1., 1., 0.)))
            .with(Rotation::from(Quat::from_z(0_f32.to_radians())))
            .with(AngularVelocity::from(Quat::from_z(FRAC_PI_2)))
            .with_child(Child::build())
    }
}

struct Child;

#[entity]
impl Child {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::default()))
            .with(RelativePosition::from(Vec3::xyz(0.5, 0.5, 0.5)))
            .with(Size::from(Vec3::xyz(0.1, 0.2, 1.)))
            .with(ShapeColor::from(Color::rgb(1., 0., 1.)))
            .with(Rotation::from(Quat::from_z(0.)))
            .with(RelativeRotation::from(Quat::from_z(0_f32.to_radians())))
            .with(RelativeAngularVelocity::from(Quat::from_z(FRAC_PI_2)))
    }
}
