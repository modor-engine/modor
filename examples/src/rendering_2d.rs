use instant::Instant;
use modor::log::{info, Level};
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::{Color, DefaultMaterial2D, IntoMat, Mat, Model2D, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::Delta;
use rand::Rng;
use std::time::Duration;

const SPRITE_COUNT: usize = 1_000;
const COLORS: [Color; 10] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::WHITE,
    Color::YELLOW,
    Color::CYAN,
    Color::PURPLE,
    Color::MAROON,
    Color::GRAY,
    Color::OLIVE,
];

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Visit)]
struct Root {
    objects: Vec<Object>,
    last_frame_instant: Instant,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.get_mut::<Window>().title = "Rendering 2D".into();
        Self {
            objects: (0..SPRITE_COUNT)
                .map(|index| Object::new(ctx, index))
                .collect(),
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
    materials: Vec<Mat<DefaultMaterial2D>>,
}

impl RootNode for Resources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            materials: COLORS
                .iter()
                .map(|&color| {
                    DefaultMaterial2D::new(ctx)
                        .with_color(color)
                        .with_is_ellipse(true)
                        .into_mat(ctx, "color")
                })
                .collect(),
        }
    }
}

#[derive(Visit)]
struct Object {
    model: Model2D<DefaultMaterial2D>,
    next_update: Instant,
    velocity: Vec2,
}

impl Node for Object {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        if Instant::now() > self.next_update {
            let mut rng = rand::thread_rng();
            self.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
                .with_magnitude(0.05)
                .unwrap_or(Vec2::ZERO);
            self.next_update = Instant::now() + Duration::from_millis(rng.gen_range(200..400));
        }
        let delta = ctx.get_mut::<Delta>().duration.as_secs_f32();
        self.model.position += self.velocity * delta;
    }
}

impl Object {
    fn new(ctx: &mut Context<'_>, index: usize) -> Self {
        let mut rng = rand::thread_rng();
        let material = ctx.get_mut::<Resources>().materials[index % COLORS.len()].glob();
        let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
        let model = Model2D::new(ctx, material)
            .with_position(position)
            .with_size(Vec2::ONE * 0.01)
            .with_z_index(rng.gen_range(i16::MIN..i16::MAX));
        Self {
            model,
            next_update: Instant::now(),
            velocity: Vec2::ONE * 0.0001,
        }
    }
}
