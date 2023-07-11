#![allow(missing_docs)]

use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics::{Camera2D, Color, Material, Model, RenderTarget, Texture, Window, ZIndex2D};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, Transform2D};
use modor_resources::ResKey;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
const BACKGROUND_TEXTURE: ResKey<Texture> = ResKey::new("background");
const SMILEY_TEXTURE: ResKey<Texture> = ResKey::new("smiley");
const BACKGROUND_MATERIAL: ResKey<Material> = ResKey::new("background");
const YELLOW_SMILEY_MATERIAL: ResKey<Material> = ResKey::new("yellow-smiley");
const GREEN_SMILEY_MATERIAL: ResKey<Material> = ResKey::new("green-smiley");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics::module())
        .with_entity(textures())
        .with_entity(materials())
        .with_entity(window())
        .with_entity(background())
        .with_entity(smiley(
            GREEN_SMILEY_MATERIAL,
            Vec2::new(0.25, -0.25),
            1,
            Vec2::new(0.3, -0.8),
            FRAC_PI_2,
        ))
        .with_entity(smiley(
            YELLOW_SMILEY_MATERIAL,
            Vec2::new(-0.25, 0.25),
            2,
            Vec2::new(0.5, -0.4),
            FRAC_PI_4,
        ))
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity {
    let target_key = ResKey::unique("window");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Window::default())
        .component(Camera2D::new(CAMERA, target_key))
}

fn textures() -> impl BuiltEntity {
    let background_data = include_bytes!("../assets/background.png");
    EntityBuilder::new()
        .child_component(Texture::from_file(BACKGROUND_TEXTURE, background_data))
        .child_component(Texture::from_path(SMILEY_TEXTURE, "smiley.png"))
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Material::new(BACKGROUND_MATERIAL))
        .with(|m| m.texture_key = Some(BACKGROUND_TEXTURE))
        .child_component(Material::new(YELLOW_SMILEY_MATERIAL))
        .with(|m| m.texture_key = Some(SMILEY_TEXTURE))
        .with(|m| m.color = Color::WHITE.with_alpha(0.7))
        .child_component(Material::new(GREEN_SMILEY_MATERIAL))
        .with(|m| m.texture_key = Some(SMILEY_TEXTURE))
        .with(|m| m.color = Color::CYAN)
}

fn background() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .component(Model::rectangle(BACKGROUND_MATERIAL, CAMERA))
}

fn smiley(
    material_key: ResKey<Material>,
    position: Vec2,
    z_index: u16,
    velocity: Vec2,
    angular_velocity: f32,
) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = position)
        .with(|t| *t.size = Vec2::new(0.2, 0.2))
        .component(Dynamics2D::new())
        .with(|d| *d.velocity = velocity)
        .with(|d| *d.angular_velocity = angular_velocity)
        .component(Model::rectangle(material_key, CAMERA))
        .component(ZIndex2D::from(z_index))
        .component(Smiley)
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
