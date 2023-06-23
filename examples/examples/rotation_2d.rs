#![allow(missing_docs)]

use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics_new2::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, RelativeTransform2D, Transform2D};
use std::f32::consts::FRAC_PI_2;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_text::module())
        .with_entity(Material::new(MaterialKey::Object).with_color(Color::YELLOW))
        .with_entity(Material::new(MaterialKey::Child).with_color(Color::MAGENTA))
        .with_entity(window())
        .with_entity(object())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default())
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn object() -> impl BuiltEntity {
    let position = Vec2::new(0.15, 0.15);
    let size = Vec2::new(0.25, 0.5);
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position).with_size(size))
        .with(Dynamics2D::new().with_angular_velocity(FRAC_PI_2))
        .with(Model::rectangle(MaterialKey::Object).with_camera_key(CameraKey))
        .with_child(child())
}

fn child() -> impl BuiltEntity {
    let size = Vec2::new(0.1, 0.2);
    let relative_position = Vec2::ONE * 0.5;
    EntityBuilder::new()
        .with(Transform2D::new().with_size(size))
        .with(
            RelativeTransform2D::new()
                .with_position(relative_position)
                .with_rotation(0.),
        )
        .with(Model::rectangle(MaterialKey::Child).with_camera_key(CameraKey))
        .with(ZIndex2D::from(1))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum MaterialKey {
    Object,
    Child,
}
