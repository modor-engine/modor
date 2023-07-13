#![allow(missing_docs)]

use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics::{window_target, Color, Material, Model, ZIndex2D, WINDOW_CAMERA_2D};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, RelativeTransform2D, Transform2D};
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_2;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(object())
        .run(modor_graphics::runner);
}

fn object() -> impl BuiltEntity {
    let material_key = ResKey::unique("object");
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = Vec2::new(0.15, 0.15))
        .with(|t| *t.size = Vec2::new(0.25, 0.5))
        .component(Dynamics2D::new())
        .with(|d| *d.angular_velocity = FRAC_PI_2)
        .component(Model::rectangle(material_key, WINDOW_CAMERA_2D))
        .component(Material::new(material_key))
        .with(|m| m.color = Color::YELLOW)
        .child_entity(child())
}

fn child() -> impl BuiltEntity {
    let material_key = ResKey::unique("child");
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(0.1, 0.2))
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::ONE * 0.5))
        .with(|t| t.rotation = Some(0.))
        .component(Model::rectangle(material_key, WINDOW_CAMERA_2D))
        .component(ZIndex2D::from(1))
        .component(Material::new(material_key))
        .with(|m| m.color = Color::MAGENTA)
}
