#![allow(missing_docs, clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, EntityBuilder, Single, SingletonComponent};
use modor_graphics_new2::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, PhysicsModule, Transform2D};
use rand::Rng;
use std::time::Duration;

const SPRITE_COUNT: usize = 1000;
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

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics_new2::module())
        .with_entity(FpsPrinter)
        .with_entity(window())
        .with_entity(materials())
        .with_entity(sprites())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default())
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new().with_children(|b| {
        for (color_id, color) in COLORS.into_iter().enumerate() {
            b.add(Material::ellipse(MaterialKey(color_id)).with_color(color));
        }
    })
}

fn sprites() -> impl BuiltEntity {
    EntityBuilder::new().with_children(move |b| {
        for entity_id in 0..SPRITE_COUNT {
            b.add(sprite(entity_id));
        }
    })
}

fn sprite(entity_id: usize) -> impl BuiltEntity {
    let mut rng = rand::thread_rng();
    let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
    let size = Vec2::ONE * 0.01;
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position).with_size(size))
        .with(Dynamics2D::new())
        .with(Model::rectangle(MaterialKey(entity_id % COLORS.len())).with_camera_key(CameraKey))
        .with(ZIndex2D::from(rng.gen_range(0..u16::MAX)))
        .with(RandomMovement::new())
}

#[derive(Component)]
struct RandomMovement {
    next_update: Instant,
}

#[systems]
impl RandomMovement {
    fn new() -> Self {
        Self {
            next_update: Instant::now(),
        }
    }

    #[run]
    fn update_velocity(&mut self, dynamics: &mut Dynamics2D) {
        if Instant::now() > self.next_update {
            let mut rng = rand::thread_rng();
            *dynamics.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
                .with_magnitude(0.05)
                .unwrap_or(Vec2::ZERO);
            self.next_update = Instant::now() + Duration::from_millis(rng.gen_range(100..200));
        }
    }
}

#[derive(SingletonComponent)]
struct FpsPrinter;

#[systems]
impl FpsPrinter {
    #[run]
    fn run(delta: Single<'_, DeltaTime>) {
        log::warn!("FPS: {}", (1. / delta.get().as_secs_f32()).round() as u32);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct MaterialKey(usize);
