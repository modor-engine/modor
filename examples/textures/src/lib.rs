#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use modor::{entity, singleton, App, Built, EntityBuilder};
use modor_graphics::{
    Color, GraphicsModule, Mesh2D, Texture, TextureConfig, TextureRef, WindowSettings,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default().title("Modor - textures"),
        ))
        .with_entity(Texture::build(AppTextureRef::Background))
        .with_entity(Texture::build(AppTextureRef::Smiley))
        .with_entity(Background::build())
        .with_entity(Smiley::build(
            Vec2::new(0.25, -0.25),
            1.,
            Vec2::new(0.3, -0.8),
            FRAC_PI_2,
            Color::GREEN,
            Color::CYAN,
        ))
        .with_entity(Smiley::build(
            Vec2::new(-0.25, 0.25),
            2.,
            Vec2::new(0.5, -0.4),
            FRAC_PI_4,
            Color::YELLOW.with_alpha(0.7),
            Color::WHITE.with_alpha(0.7),
        ))
        .run(modor_graphics::runner);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum AppTextureRef {
    Background,
    Smiley,
}

impl TextureRef for AppTextureRef {
    fn config(&self) -> TextureConfig {
        let path = match self {
            Self::Background => "background.png",
            Self::Smiley => "smiley.png",
        };
        TextureConfig::from_path(path).with_smooth(true)
    }
}

struct Background;

#[singleton]
impl Background {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).with(Transform2D::new()).with(
            Mesh2D::rectangle()
                .with_color(Color::rgb(0.3, 0.4, 0.6))
                .with_texture(AppTextureRef::Background),
        )
    }
}

struct Smiley;

#[entity]
impl Smiley {
    const SIZE: Vec2 = Vec2::new(0.2, 0.2);

    fn build(
        position: Vec2,
        z: f32,
        velocity: Vec2,
        angular_velocity: f32,
        color: Color,
        texture_color: Color,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Self::SIZE)
                    .with_rotation(FRAC_PI_4),
            )
            .with(
                Dynamics2D::new()
                    .with_velocity(velocity)
                    .with_angular_velocity(angular_velocity),
            )
            .with(
                Mesh2D::ellipse()
                    .with_z(z)
                    .with_color(color)
                    .with_texture_color(texture_color)
                    .with_texture(AppTextureRef::Smiley),
            )
    }

    #[run]
    fn bounce(transform: &mut Transform2D, dynamics: &mut Dynamics2D) {
        if transform.position.x < -0.5 + Self::SIZE.x / 2. {
            dynamics.velocity.x *= -1.;
            transform.position.x = -0.5 + Self::SIZE.x / 2.;
        }
        if transform.position.x > 0.5 - Self::SIZE.x / 2. {
            dynamics.velocity.x *= -1.;
            transform.position.x = 0.5 - Self::SIZE.x / 2.;
        }
        if transform.position.y < -0.5 + Self::SIZE.y / 2. {
            dynamics.velocity.y *= -1.;
            transform.position.y = -0.5 + Self::SIZE.y / 2.;
        }
        if transform.position.y > 0.5 - Self::SIZE.y / 2. {
            dynamics.velocity.y *= -1.;
            transform.position.y = 0.5 - Self::SIZE.y / 2.;
        }
    }
}
