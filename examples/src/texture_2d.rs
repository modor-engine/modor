use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics::{
    instance_2d, window_target, Color, Material, Texture, ZIndex2D, WINDOW_CAMERA_2D,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};
use modor_resources::ResKey;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window_target())
        .with_entity(background())
        .with_entity(smileys())
        .run(modor_graphics::runner);
}

fn background() -> impl BuiltEntity {
    let texture_key = ResKey::unique("background");
    let background_data = include_bytes!("../assets/background.png");
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|m: &mut Material| m.texture_key = Some(texture_key))
        .component(Texture::from_file(texture_key, background_data))
}

fn smileys() -> impl BuiltEntity {
    let texture_key = ResKey::unique("smiley");
    EntityBuilder::new()
        .component(Texture::from_path(texture_key, "smiley.png"))
        .child_entity(smiley(
            texture_key,
            Color::CYAN,
            Vec2::new(0.25, -0.25),
            1,
            Vec2::new(0.3, -0.8),
            FRAC_PI_2,
        ))
        .child_entity(smiley(
            texture_key,
            Color::WHITE.with_alpha(0.7),
            Vec2::new(-0.25, 0.25),
            2,
            Vec2::new(0.5, -0.4),
            FRAC_PI_4,
        ))
}

fn smiley(
    texture_key: ResKey<Texture>,
    color: Color,
    position: Vec2,
    z_index: u16,
    velocity: Vec2,
    angular_velocity: f32,
) -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.2, 0.2))
        .updated(|m: &mut Material| m.texture_key = Some(texture_key))
        .updated(|m: &mut Material| m.color = color)
        .component(Dynamics2D::new())
        .with(|d| d.velocity = velocity)
        .with(|d| d.angular_velocity = angular_velocity)
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
