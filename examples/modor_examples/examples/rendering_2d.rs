//! Example of 2D rendering.
#![allow(clippy::cast_precision_loss, clippy::print_stdout)]

use instant::Instant;
use modor::{entity, singleton, App, Built, EntityBuilder, Single};
use modor_graphics::{
    Camera2D, Color, FrameRate, FrameRateLimit, GraphicsModule, ShapeColor, SurfaceSize,
};
use modor_physics::{DeltaTime, Position, Scale, Shape, Velocity};
use rand::prelude::ThreadRng;
use rand::Rng;
use std::time::Duration;

const TITLE: &str = "Modor - rendering 2D";

fn main() {
    App::new()
        .with_entity(GraphicsModule::build(SurfaceSize::new(800, 600), TITLE))
        .with_entity(MainModule::build(1000))
        .run(modor_graphics::runner);
}

struct MainModule;

#[singleton]
impl MainModule {
    fn build(entity_count: usize) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with_child(FrameRateLimit::build(FrameRate::VSync))
            .with_child(Camera2D::build(Position::xy(0., 0.), Scale::xy(1.5, 1.5)))
            .with_child(FrameRateDisplay::build())
            .with_children(move |b| {
                for _ in 0..entity_count {
                    b.add(Sprite::build());
                }
            })
    }
}

struct Sprite {
    next_update: Instant,
}

#[entity]
impl Sprite {
    fn build() -> impl Built<Self> {
        let mut rng = rand::thread_rng();
        EntityBuilder::new(Self {
            next_update: Instant::now(),
        })
        .with(Position::xy(
            Self::random_f32(&mut rng),
            Self::random_f32(&mut rng),
        ))
        .with(Scale::xy(0.01, 0.01))
        .with(Velocity::xy(0., 0.))
        .with(Shape::Circle2D)
        .with(ShapeColor(Color::rgb(
            Self::random_f32(&mut rng) + 0.5,
            Self::random_f32(&mut rng) + 0.5,
            Self::random_f32(&mut rng) + 0.5,
        )))
    }

    #[run]
    fn update_velocity(&mut self, velocity: &mut Velocity) {
        if Instant::now() > self.next_update {
            let mut rng = rand::thread_rng();
            velocity.x = Self::random_f32(&mut rng);
            velocity.y = Self::random_f32(&mut rng);
            velocity.set_magnitude(0.05);
            self.next_update = Instant::now() + Duration::from_millis(200);
        }
    }

    fn random_f32(rng: &mut ThreadRng) -> f32 {
        // Random number between -0.5 and 0.5
        (rng.gen_range(-1_000_000..1_000_000) as f32) / 2_000_000.
    }
}

struct FrameRateDisplay;

#[singleton]
impl FrameRateDisplay {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn update_frame_rate(delta_time: Single<'_, DeltaTime>) {
        let fps = (1. / delta_time.get().as_secs_f32()).round();
        println!("FPS: {}", fps);
    }
}
