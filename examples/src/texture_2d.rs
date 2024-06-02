use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::Res;
use modor_graphics::{Color, DefaultMaterial2D, Mat, Model2D, Texture};
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
            background_texture: Res::from_path(ctx, "background", "background.png"),
            smiley_texture: Res::from_path(ctx, "smiley", "smiley.png"),
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
        let mut material_data = DefaultMaterial2D::new(ctx);
        material_data.texture = ctx.get_mut::<Resources>().background_texture.glob().clone();
        let material = Mat::new(ctx, "background", material_data);
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
        let mut material_data = DefaultMaterial2D::new(ctx);
        material_data.texture = ctx.get_mut::<Resources>().smiley_texture.glob().clone();
        material_data.color = color;
        let material = Mat::new(ctx, "smiley", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.position = position;
        model.size *= 0.2;
        model.z_index = z_index;
        Self {
            material,
            model,
            velocity,
            angular_velocity,
        }
    }
}
