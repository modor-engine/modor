#![allow(missing_docs)]

use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics_new2::{
    Camera2D, Color, Material, Model, RenderTarget, Texture, Window, ZIndex2D,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use std::fmt::Debug;
use std::hash::Hash;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics_new2::module())
        .with_entity(Texture::from_file(
            TextureKey::Background,
            include_bytes!("../assets/background.png"),
        ))
        .with_entity(Texture::from_path(TextureKey::Smiley, "smiley.png"))
        .with_entity(
            Material::new(MaterialKey::Background).with_texture_key(TextureKey::Background),
        )
        .with_entity(
            Material::new(MaterialKey::YellowSmiley)
                .with_texture_key(TextureKey::Smiley)
                .with_color(Color::WHITE.with_alpha(0.7)),
        )
        .with_entity(
            Material::new(MaterialKey::GreenSmiley)
                .with_texture_key(TextureKey::Smiley)
                .with_color(Color::CYAN),
        )
        .with_entity(window())
        .with_entity(background())
        .with_entity(smiley(
            MaterialKey::GreenSmiley,
            Vec2::new(0.25, -0.25),
            1,
            Vec2::new(0.3, -0.8),
            FRAC_PI_2,
        ))
        .with_entity(smiley(
            MaterialKey::YellowSmiley,
            Vec2::new(-0.25, 0.25),
            2,
            Vec2::new(0.5, -0.4),
            FRAC_PI_4,
        ))
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default())
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn background() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(Model::rectangle(MaterialKey::Background).with_camera_key(CameraKey))
}

fn smiley(
    material_key: MaterialKey,
    position: Vec2,
    z_index: u16,
    velocity: Vec2,
    angular_velocity: f32,
) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.2, 0.2)),
        )
        .with(
            Dynamics2D::new()
                .with_velocity(velocity)
                .with_angular_velocity(angular_velocity),
        )
        .with(Model::rectangle(material_key).with_camera_key(CameraKey))
        .with(ZIndex2D::from(z_index))
        .with(Smiley)
}

#[derive(Component)]
struct Smiley;

#[systems]
impl Smiley {
    #[run]
    fn bounce(transform: &mut Transform2D, dynamics: &mut Dynamics2D) {
        if transform.position.x < -0.5 + transform.size.x / 2. {
            dynamics.velocity.x *= -1.;
            transform.position.x = -0.5 + transform.size.x / 2.;
        }
        if transform.position.x > 0.5 - transform.size.x / 2. {
            dynamics.velocity.x *= -1.;
            transform.position.x = 0.5 - transform.size.x / 2.;
        }
        if transform.position.y < -0.5 + transform.size.y / 2. {
            dynamics.velocity.y *= -1.;
            transform.position.y = -0.5 + transform.size.y / 2.;
        }
        if transform.position.y > 0.5 - transform.size.y / 2. {
            dynamics.velocity.y *= -1.;
            transform.position.y = 0.5 - transform.size.y / 2.;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum MaterialKey {
    Background,
    YellowSmiley,
    GreenSmiley,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum TextureKey {
    Background,
    Smiley,
}
