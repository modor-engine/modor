use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Color, DefaultMaterial2D, IntoMat, Mat, Model2D, Texture};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    background: Background,
    smileys: Vec<Smiley>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            background: Background::new(ctx),
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
            background_texture: Texture::new(ctx, "background").load_from_path("background.png"),
            smiley_texture: Texture::new(ctx, "smiley").load_from_path("smiley.png"),
        }
    }
}

#[derive(Node, Visit)]
struct Background {
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
}

impl Background {
    fn new(ctx: &mut Context<'_>) -> Self {
        let material = DefaultMaterial2D::new(ctx)
            .with_texture(ctx.get_mut::<Resources>().background_texture.glob().clone())
            .into_mat(ctx, "background");
        let model = Model2D::new(ctx, material.glob());
        Self { material, model }
    }
}

#[derive(Visit)]
struct Smiley {
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
    velocity: Vec2,
    angular_velocity: f32,
}

impl Node for Smiley {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        if self.model.position.x < -0.5 + self.model.size.x / 2. {
            self.velocity.x *= -1.;
            self.model.position.x = -0.5 + self.model.size.x / 2.;
        }
        if self.model.position.x > 0.5 - self.model.size.x / 2. {
            self.velocity.x *= -1.;
            self.model.position.x = 0.5 - self.model.size.x / 2.;
        }
        if self.model.position.y < -0.5 + self.model.size.y / 2. {
            self.velocity.y *= -1.;
            self.model.position.y = -0.5 + self.model.size.y / 2.;
        }
        if self.model.position.y > 0.5 - self.model.size.y / 2. {
            self.velocity.y *= -1.;
            self.model.position.y = 0.5 - self.model.size.y / 2.;
        }
        self.model.position += self.velocity / 60.;
        self.model.rotation += self.angular_velocity / 60.;
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
        let material = DefaultMaterial2D::new(ctx)
            .with_color(color)
            .with_texture(ctx.get_mut::<Resources>().smiley_texture.glob().clone())
            .into_mat(ctx, "smiley");
        let model = Model2D::new(ctx, material.glob())
            .with_position(position)
            .with_size(Vec2::ONE * 0.2)
            .with_z_index(z_index);
        Self {
            material,
            model,
            velocity,
            angular_velocity,
        }
    }
}
