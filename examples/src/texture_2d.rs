use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Color, Sprite2D, Texture};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    background: Sprite2D,
    smileys: Vec<Smiley>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let background_texture = ctx.get_mut::<Resources>().background_texture.glob().clone();
        Self {
            background: Sprite2D::new(ctx, "background")
                .with_material(|m| m.texture = background_texture),
            smileys: vec![
                Smiley::new(
                    ctx,
                    Color::CYAN,
                    Vec2::new(0.25, -0.25),
                    1,
                    Vec2::new(0.3, -0.8),
                    FRAC_PI_2,
                ),
                Smiley::new(
                    ctx,
                    Color::WHITE.with_alpha(0.7),
                    Vec2::new(-0.25, 0.25),
                    2,
                    Vec2::new(0.5, -0.4),
                    FRAC_PI_4,
                ),
            ],
        }
    }
}

#[derive(Node, Visit)]
struct Resources {
    background_texture: Res<Texture>,
    smiley_texture: Res<Texture>,
}

impl RootNode for Resources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            background_texture: Texture::new(ctx, "background")
                .load_from_path(ctx, "background.png"),
            smiley_texture: Texture::new(ctx, "smiley").load_from_path(ctx, "smiley.png"),
        }
    }
}

#[derive(Visit)]
struct Smiley {
    sprite: Sprite2D,
    velocity: Vec2,
    angular_velocity: f32,
}

impl Node for Smiley {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        let model = &mut self.sprite.model;
        if model.position.x < -0.5 + model.size.x / 2. {
            self.velocity.x *= -1.;
            model.position.x = -0.5 + model.size.x / 2.;
        }
        if model.position.x > 0.5 - model.size.x / 2. {
            self.velocity.x *= -1.;
            model.position.x = 0.5 - model.size.x / 2.;
        }
        if model.position.y < -0.5 + model.size.y / 2. {
            self.velocity.y *= -1.;
            model.position.y = -0.5 + model.size.y / 2.;
        }
        if model.position.y > 0.5 - model.size.y / 2. {
            self.velocity.y *= -1.;
            model.position.y = 0.5 - model.size.y / 2.;
        }
        model.position += self.velocity / 60.;
        model.rotation += self.angular_velocity / 60.;
    }
}

impl Smiley {
    fn new(
        ctx: &mut Context<'_>,
        color: Color,
        position: Vec2,
        z_index: i16,
        velocity: Vec2,
        angular_velocity: f32,
    ) -> Self {
        let texture = ctx.get_mut::<Resources>().smiley_texture.glob().clone();
        Self {
            sprite: Sprite2D::new(ctx, "smiley")
                .with_model(|m| m.position = position)
                .with_model(|m| m.size = Vec2::ONE * 0.2)
                .with_model(|m| m.z_index = z_index)
                .with_material(|m| m.color = color)
                .with_material(|m| m.texture = texture),
            velocity,
            angular_velocity,
        }
    }
}
