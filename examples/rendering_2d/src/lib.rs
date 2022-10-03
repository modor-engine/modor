#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

use instant::Instant;
use modor::{entity, singleton, App, Built, EntityBuilder, Single};
use modor_graphics::{Camera2D, Color, GraphicsModule, Mesh2D, SurfaceSize, WindowSettings};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, Transform2D};
use rand::prelude::ThreadRng;
use rand::Rng;
use std::time::Duration;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default()
                .size(SurfaceSize::new(800, 600))
                .title("Modor - rendering 2D"),
        ))
        .with_entity(MainModule::build(10000))
        .run(modor_graphics::runner);
}

struct MainModule;

#[singleton]
impl MainModule {
    fn build(entity_count: usize) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with_child(Camera2D::build(Vec2::new(0., 0.), Vec2::new(1.5, 1.5)))
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
        .with(
            Transform2D::new()
                .with_position(Vec2::new(
                    Self::random_f32(&mut rng),
                    Self::random_f32(&mut rng),
                ))
                .with_size(Vec2::ONE * 0.01),
        )
        .with(Dynamics2D::new())
        .with(Mesh2D::ellipse().with_color(Color::rgb(
            Self::random_f32(&mut rng) + 0.5,
            Self::random_f32(&mut rng) + 0.5,
            Self::random_f32(&mut rng) + 0.5,
        )))
    }

    #[run]
    fn update_velocity(&mut self, dynamics: &mut Dynamics2D) {
        if Instant::now() > self.next_update {
            let mut rng = rand::thread_rng();
            *dynamics.velocity = Vec2::new(Self::random_f32(&mut rng), Self::random_f32(&mut rng))
                .with_magnitude(0.05)
                .unwrap_or(Vec2::ZERO);
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
