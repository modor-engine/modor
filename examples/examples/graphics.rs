#![allow(missing_docs)]

use instant::Instant;
use modor::log::{info, Level};
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_resources::Res;
use modor_graphics::{Color, DefaultMaterial2D, Mat, Model2D, Texture, Window};

fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Visit)]
struct Root {
    smiley: Vec<Smiley>,
    last_frame_instant: Instant,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let window = ctx.root::<Window>().get_mut(ctx);
        window.title = "Example".into();
        window.target.background_color = Color::GRAY;
        Self {
            smiley: (0..1000).map(|_| Smiley::new(ctx)).collect(),
            last_frame_instant: Instant::now(),
        }
    }
}

impl Node for Root {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        let now = Instant::now();
        info!(
            "FPS: {}",
            1. / (now - self.last_frame_instant).as_secs_f32()
        );
        self.last_frame_instant = now;
    }
}

#[derive(Node, Visit)]
struct Resources {
    texture: Res<Texture>,
}

impl RootNode for Resources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            texture: Res::<Texture>::from_path(ctx, "smiley", "smiley.png"),
        }
    }
}

#[derive(Node, Visit)]
struct Smiley {
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
}

impl Smiley {
    fn new(ctx: &mut Context<'_>) -> Self {
        let texture = ctx.root::<Resources>().get(ctx).texture.glob().clone();
        let mut material = Mat::new(ctx, "smiley", DefaultMaterial2D::default());
        material.texture = Some(texture);
        let mut model = Model2D::new(ctx);
        model.material = material.glob();
        Self { material, model }
    }
}
