use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Color, Sprite2D, Texture};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    background: Sprite2D,
    smileys: Vec<Smiley>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let background_texture = app
            .get_mut::<Resources>()
            .background_texture
            .glob()
            .to_ref();
        Self {
            background: Sprite2D::new(app).with_material(|m| m.texture = background_texture),
            smileys: vec![
                Smiley::new(
                    app,
                    Color::CYAN,
                    Vec2::new(0.25, -0.25),
                    1,
                    Vec2::new(0.3, -0.8),
                    FRAC_PI_2,
                ),
                Smiley::new(
                    app,
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

impl State for Root {
    fn update(&mut self, app: &mut App) {
        self.background.update(app);
        for smiley in &mut self.smileys {
            smiley.update(app);
        }
    }
}

#[derive(FromApp)]
struct Resources {
    background_texture: Glob<Res<Texture>>,
    smiley_texture: Glob<Res<Texture>>,
}

impl State for Resources {
    fn init(&mut self, app: &mut App) {
        self.background_texture
            .updater()
            .path("background.png")
            .resource(|r| r.is_smooth(false))
            .resource(|r| r.is_repeated(true))
            .apply(app);
    }
}

struct Smiley {
    sprite: Sprite2D,
    velocity: Vec2,
    angular_velocity: f32,
}

impl Smiley {
    fn new(
        app: &mut App,
        color: Color,
        position: Vec2,
        z_index: i16,
        velocity: Vec2,
        angular_velocity: f32,
    ) -> Self {
        let texture = app.get_mut::<Resources>().smiley_texture.glob().to_ref();
        Self {
            sprite: Sprite2D::new(app)
                .with_model(|m| m.position = position)
                .with_model(|m| m.size = Vec2::ONE * 0.2)
                .with_model(|m| m.z_index = z_index)
                .with_material(|m| m.color = color)
                .with_material(|m| m.texture = texture),
            velocity,
            angular_velocity,
        }
    }

    fn update(&mut self, app: &mut App) {
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
        self.sprite.update(app);
    }
}
