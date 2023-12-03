use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, EntityBuilder, SingleRef, SingletonComponent};
use modor_graphics::{
    instance_2d, window_target, Color, Material, MaterialType, ZIndex2D, WINDOW_CAMERA_2D,
};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, Transform2D};
use modor_resources::IndexResKey;
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

const MATERIAL: IndexResKey<Material> = IndexResKey::new("sprite");

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(FpsPrinter)
        .with_entity(window_target())
        .with_entity(materials())
        .with_entity(sprites())
        .run(modor_graphics::runner);
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new().child_entities(|g| {
        for (color_id, color) in COLORS.into_iter().enumerate() {
            let mut material = Material::ellipse(MATERIAL.get(color_id));
            material.color = color;
            g.add(material);
        }
    })
}

fn sprites() -> impl BuiltEntity {
    EntityBuilder::new().child_entities(move |b| {
        for entity_id in 0..SPRITE_COUNT {
            b.add(sprite(entity_id));
        }
    })
}

fn sprite(entity_id: usize) -> impl BuiltEntity {
    let mut rng = rand::thread_rng();
    let material_key = MATERIAL.get(entity_id % COLORS.len());
    let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
    instance_2d(WINDOW_CAMERA_2D, MaterialType::Key(material_key))
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.01)
        .component(Dynamics2D::new())
        .component(ZIndex2D::from(rng.gen_range(0..u16::MAX)))
        .component(RandomMovement::new())
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
            dynamics.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
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
    fn run(delta: SingleRef<'_, '_, DeltaTime>) {
        log::warn!(
            "FPS: {}",
            (1. / delta.get().get().as_secs_f32()).round() as u32
        );
    }
}
