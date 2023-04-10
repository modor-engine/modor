use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, EntityBuilder, Single, SingletonComponent};
use modor_graphics_new2::{Camera2D, Color, Material, Model, RenderTarget, Window, ZIndex2D};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, PhysicsModule, Transform2D};
use rand::Rng;
use std::time::Duration;

// TODO: remove this example -> create bench

const SPRITE_MATERIAL_COUNT: u32 = 10;

pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics_new2::module())
        // .with_entity(FpsPrinter)
        .with_entity(window())
        .with_entity(materials())
        .with_entity(sprites(10_000))
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default().with_cursor_shown(false))
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(Material::ellipse(MaterialKey::Sprite(0)).with_color(Color::RED))
        .with_child(Material::ellipse(MaterialKey::Sprite(1)).with_color(Color::GREEN))
        .with_child(Material::ellipse(MaterialKey::Sprite(2)).with_color(Color::BLUE))
        .with_child(Material::ellipse(MaterialKey::Sprite(3)).with_color(Color::WHITE))
        .with_child(Material::ellipse(MaterialKey::Sprite(4)).with_color(Color::YELLOW))
        .with_child(Material::ellipse(MaterialKey::Sprite(5)).with_color(Color::CYAN))
        .with_child(Material::ellipse(MaterialKey::Sprite(6)).with_color(Color::PURPLE))
        .with_child(Material::ellipse(MaterialKey::Sprite(7)).with_color(Color::MAROON))
        .with_child(Material::ellipse(MaterialKey::Sprite(8)).with_color(Color::GRAY))
        .with_child(Material::ellipse(MaterialKey::Sprite(9)).with_color(Color::OLIVE))
}

fn sprites(entity_count: u32) -> impl BuiltEntity {
    EntityBuilder::new().with_children(move |b| {
        for entity_id in 0..entity_count {
            b.add(sprite(entity_id));
        }
    })
}

fn sprite(entity_id: u32) -> impl BuiltEntity {
    let mut rng = rand::thread_rng();
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::new(
                    rng.gen_range(-0.2..0.2),
                    rng.gen_range(-0.2..0.2),
                ))
                .with_size(Vec2::ONE * 0.01),
        )
        .with(Dynamics2D::new())
        // .with_option((entity_id % 1 == 0).then(Dynamics2D::new))
        .with(
            Model::rectangle(MaterialKey::Sprite(entity_id % SPRITE_MATERIAL_COUNT))
                .with_camera_key(CameraKey),
        )
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
enum MaterialKey {
    Sprite(u32),
}
