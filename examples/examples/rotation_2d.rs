#![allow(missing_docs)]

use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, RelativeTransform2D, Transform2D};
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_2;

const CAMERA: ResKey<Camera2D> = ResKey::new("main");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_text::module())
        .with_entity(window())
        .with_entity(object())
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity {
    let target_key = ResKey::unique("window");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Window::default())
        .component(Camera2D::new(CAMERA, target_key))
}

fn object() -> impl BuiltEntity {
    let position = Vec2::new(0.15, 0.15);
    let size = Vec2::new(0.25, 0.5);
    let material_key = ResKey::unique("object");
    EntityBuilder::new()
        .component(Transform2D::new().with_position(position).with_size(size))
        .component(Dynamics2D::new().with_angular_velocity(FRAC_PI_2))
        .component(Model::rectangle(material_key, CAMERA))
        .component(Material::new(material_key).with_color(Color::YELLOW))
        .child_entity(child())
}

fn child() -> impl BuiltEntity {
    let size = Vec2::new(0.1, 0.2);
    let relative_position = Vec2::ONE * 0.5;
    let material_key = ResKey::unique("child");
    EntityBuilder::new()
        .component(Transform2D::new().with_size(size))
        .component(
            RelativeTransform2D::new()
                .with_position(relative_position)
                .with_rotation(0.),
        )
        .component(Model::rectangle(material_key, CAMERA))
        .component(ZIndex2D::from(1))
        .component(Material::new(material_key).with_color(Color::MAGENTA))
}
