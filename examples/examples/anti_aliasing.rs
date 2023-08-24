#![allow(missing_docs)]

use modor::{systems, App, BuiltEntity, EntityBuilder, SingleMut, SingleRef, SingletonComponent};
use modor_graphics::{
    window_target, AntiAliasing, Color, Material, Model, ZIndex2D, WINDOW_CAMERA_2D,
};
use modor_input::{InputModule, Key, Keyboard};
use modor_math::Vec2;
use modor_physics::{PhysicsModule, Transform2D};
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_8;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(InputModule::build())
        .with_entity(modor_text::module())
        .with_entity(AntiAliasing::Smaa(8))
        .with_entity(AntiAliasingController)
        .with_entity(
            window_target()
                .component(Transform2D::new())
                .with(|t| *t.size = Vec2::ONE * 0.3),
        )
        .with_entity(information())
        .with_entity(object())
        .run(modor_graphics::runner);
}

fn object() -> impl BuiltEntity {
    let material_key = ResKey::unique("object");
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::ONE * 0.5)
        .with(|t| *t.rotation = FRAC_PI_8)
        .component(Model::rectangle(material_key, WINDOW_CAMERA_2D))
        .component(Material::new(material_key))
        .with(|m| m.color = Color::YELLOW)
}

fn information() -> impl BuiltEntity {
    let material_key = ResKey::unique("information");
    modor_text::text_material(
        material_key,
        "Sample count: 1\nUp arrow key: increase\nDown arrow key: decrease",
        50.,
    )
    .updated(|m: &mut Material| m.front_color = Color::WHITE)
    .updated(|m: &mut Material| m.color = Color::INVISIBLE)
    .component(Model::rectangle(material_key, WINDOW_CAMERA_2D))
    .component(Transform2D::new())
    .component(ZIndex2D::from(1))
}

#[derive(SingletonComponent)]
struct AntiAliasingController;

#[systems]
impl AntiAliasingController {
    #[run]
    fn update(
        mut anti_aliasing: SingleMut<'_, '_, AntiAliasing>,
        keyboard: SingleRef<'_, '_, Keyboard>,
    ) {
        let keyboard = keyboard.get();
        if let AntiAliasing::Smaa(sample_count) = anti_aliasing.get_mut() {
            if keyboard.key(Key::Up).is_just_released {
                *sample_count = Self::next_sample_count(*sample_count);
            }
            if keyboard.key(Key::Down).is_just_released {
                *sample_count = Self::previous_sample_count(*sample_count);
            }
            println!("Sample count: {}", sample_count);
        }
    }

    fn next_sample_count(sample_count: u32) -> u32 {
        match sample_count {
            1 => 2,
            2 => 4,
            4 => 8,
            _ => 16,
        }
    }

    fn previous_sample_count(sample_count: u32) -> u32 {
        match sample_count {
            32 => 16,
            16 => 8,
            8 => 4,
            4 => 2,
            _ => 1,
        }
    }
}
