#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, App, Built, EntityBuilder, Single};
use modor_graphics::{
    Color, FrameRate, FrameRateLimit, GraphicsModule, ShapeColor, SurfaceSize, WindowSettings,
};
use modor_math::Quaternion;
use modor_physics::{
    DeltaTime, Position, RelativePosition, RelativeRotation, RelativeSize, Rotation, RotationAxis,
    Size,
};
use std::f32::consts::FRAC_2_PI;

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
            .with(Position::xy(0.15, 0.15))
            .with(Size::xy(0.25, 0.25))
            .with(ShapeColor(Color::rgb(1., 1., 0.)))
            .with(Rotation::new(RotationAxis::Z, 0_f32.to_radians()))
            .with_child(Child::build())
    }

    #[run]
    fn rotate(rotation: &mut Rotation, delta_time: Single<'_, DeltaTime>) {
        // // TODO: fix wrong rotation direction
        *rotation = rotation.with_rotation(Rotation::new(
            RotationAxis::Z,
            delta_time.get().as_secs_f32() * FRAC_2_PI,
        ));
    }
}

struct Child;

#[entity]
impl Child {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::ZERO)
            .with(RelativePosition::xy(-0.7, -0.7))
            .with(Size::ZERO)
            .with(RelativeSize::xyz(0.5, 0.5, 1.))
            .with(ShapeColor(Color::rgb(1., 0., 1.)))
            .with(Rotation::new(RotationAxis::Z, 0.))
            .with(RelativeRotation::new(RotationAxis::Z, 0_f32.to_radians()))
    }

    // #[run]
    // fn rotate(rotation: &mut RelativeRotation, delta_time: Single<'_, DeltaTime>) {
    //     *rotation = rotation.with_rotation(RelativeRotation::new(
    //         RotationAxis::Z,
    //        - delta_time.get().as_secs_f32() * FRAC_2_PI,
    //     ));
    // }
}
