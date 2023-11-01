#![allow(missing_docs)]

use modor::{
    systems, App, BuiltEntity, NoSystem, Single, SingleMut, SingleRef, SingletonComponent,
};
use modor_graphics::{
    model_2d, window_target, AntiAliasing, Color, Material, Model2DMaterial, ZIndex2D,
    WINDOW_CAMERA_2D,
};
use modor_input::{Key, Keyboard};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{text_2d, Alignment, Text};
use std::f32::consts::FRAC_PI_8;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(AntiAliasing::default())
        .with_entity(AntiAliasingController)
        .with_entity(window_target())
        .with_entity(information())
        .with_entity(object())
        .run(modor_graphics::runner);
}

fn object() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .updated(|t: &mut Transform2D| t.rotation = FRAC_PI_8)
        .updated(|m: &mut Material| m.color = Color::YELLOW)
}

fn information() -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, "", 50.)
        .updated(|m: &mut Material| m.front_color = Color::BLACK)
        .updated(|m: &mut Material| m.color = Color::INVISIBLE)
        .updated(|t: &mut Text| t.alignment = Alignment::Left)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .component(ZIndex2D::from(1))
        .component(Information)
}

#[derive(SingletonComponent, NoSystem)]
struct Information;

#[derive(SingletonComponent)]
struct AntiAliasingController;

#[systems]
impl AntiAliasingController {
    #[run]
    fn update(
        mut anti_aliasing: SingleMut<'_, '_, AntiAliasing>,
        keyboard: SingleRef<'_, '_, Keyboard>,
        mut information: Single<'_, Information, &mut Text>,
    ) {
        let keyboard = keyboard.get();
        let anti_aliasing = anti_aliasing.get_mut();
        if keyboard[Key::Up].is_just_released() {
            Self::switch_to_next_mode(anti_aliasing);
        }
        if keyboard[Key::Down].is_just_released() {
            Self::switch_to_previous_mode(anti_aliasing);
        }
        information.get_mut().content = format!(
            "Sample count: {}\n* Up arrow key: increase\n* Down arrow key: decrease",
            anti_aliasing.mode.sample_count()
        );
    }

    fn switch_to_next_mode(anti_aliasing: &mut AntiAliasing) {
        let mode_index = anti_aliasing
            .supported_modes()
            .iter()
            .position(|a| a == &anti_aliasing.mode);
        let new_mode_index = mode_index.map_or(0, |i| {
            (i + 1).min(anti_aliasing.supported_modes().len() - 1)
        });
        anti_aliasing.mode = anti_aliasing.supported_modes()[new_mode_index];
    }

    fn switch_to_previous_mode(anti_aliasing: &mut AntiAliasing) {
        let mode_index = anti_aliasing
            .supported_modes()
            .iter()
            .position(|a| a == &anti_aliasing.mode);
        let new_mode_index = mode_index.map_or(0, |i| i.saturating_sub(1));
        anti_aliasing.mode = anti_aliasing.supported_modes()[new_mode_index];
    }
}
