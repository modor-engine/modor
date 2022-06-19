#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, App, Built, EntityBuilder, Single};
use modor_graphics::{
    Color, FrameRate, FrameRateLimit, GraphicsModule, ShapeColor, SurfaceSize, WindowSettings,
};
use modor_math::{Quat, Vec3};
use modor_physics::{
    DeltaTime, Position, RelativePosition, RelativeRotation, RelativeSize, Rotation, Size,
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
            .with(Size::from(Vec3::xy(0.25, 0.25)))
            .with(ShapeColor(Color::rgb(1., 1., 0.)))
            .with(Rotation::from(Quat::from_axis_angle(
                Vec3::Z,
                0_f32.to_radians(),
            )))
            .with_child(Child::build())
    }

    #[run]
    fn rotate(rotation: &mut Rotation, delta_time: Single<'_, DeltaTime>) {
        **rotation = rotation.with_rotation(Quat::from_axis_angle(
            Vec3::Z,
            delta_time.get().as_secs_f32() * FRAC_PI_2,
        ));
    }
}

struct Child;

#[entity]
impl Child {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::default()))
            .with(RelativePosition::from(Vec3::xy(-0.7, -0.7)))
            .with(Size::from(Vec3::default()))
            .with(RelativeSize::from(Vec3::xyz(0.5, 0.5, 1.)))
            .with(ShapeColor(Color::rgb(1., 0., 1.)))
            .with(Rotation::from(Quat::from_axis_angle(Vec3::Z, 0.)))
            .with(RelativeRotation::from(Quat::from_axis_angle(
                Vec3::Z,
                0_f32.to_radians(),
            )))
    }

    #[run]
    fn rotate(rotation: &mut RelativeRotation, delta_time: Single<'_, DeltaTime>) {
        **rotation = rotation.with_rotation(Quat::from_axis_angle(
            Vec3::Z,
            delta_time.get().as_secs_f32() * FRAC_PI_2,
        ));
    }
}
