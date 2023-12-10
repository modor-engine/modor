use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::has_pixel_diff;
use modor_graphics::{
    instance_2d, texture_target, AntiAliasing, AntiAliasingMode, Color, GraphicsModule, Material,
    Size, TextureBuffer, TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::f32::consts::FRAC_PI_8;

#[modor_test]
fn retrieve_sample_count() {
    assert_eq!(AntiAliasingMode::None.sample_count(), 1);
    assert_eq!(AntiAliasingMode::MsaaX2.sample_count(), 2);
    assert_eq!(AntiAliasingMode::MsaaX4.sample_count(), 4);
    assert_eq!(AntiAliasingMode::MsaaX8.sample_count(), 8);
    assert_eq!(AntiAliasingMode::MsaaX16.sample_count(), 16);
}

#[modor_test(disabled(macos, android, wasm))]
fn run_msaa_in_texture() {
    let mut supported_modes = vec![];
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(AntiAliasing::default())
        .with_entity(resources())
        .assert::<With<AntiAliasing>>(1, has_not_supported_modes())
        .updated()
        .assert::<With<AntiAliasing>>(1, has_supported_modes())
        .assert::<With<AntiAliasing>>(1, |e| {
            e.has(|a: &AntiAliasing| supported_modes = a.supported_modes().into())
        })
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#none", 12))
        .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX4))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#msaa_x4", 12))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#msaa_x4", 12))
        .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX16))
        .updated()
        .assert::<With<TextureBuffer>>(1, |e| {
            has_pixel_diff(
                if supported_modes.contains(&AntiAliasingMode::MsaaX16) {
                    "anti_aliasing#msaa_x16"
                } else {
                    "anti_aliasing#none"
                },
                12,
            )(e)
        })
        .with_entity(AntiAliasing::from(AntiAliasingMode::None))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#none", 12));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_entity() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX4))
        .with_entity(resources())
        .updated()
        .with_deleted_entities::<With<AntiAliasing>>()
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#none", 12));
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_graphics_module() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX4))
        .with_entity(resources())
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<AntiAliasing>>(1, has_supported_modes())
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#msaa_x4", 12));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX4))
        .with_entity(resources())
        .updated()
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .assert::<With<AntiAliasing>>(1, has_not_supported_modes())
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<AntiAliasing>>(1, has_supported_modes())
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#msaa_x4", 12));
}

assertion_functions!(
    fn has_supported_modes(anti_aliasing: &AntiAliasing) {
        assert!(anti_aliasing.supported_modes().len() >= 2);
        assert_eq!(anti_aliasing.supported_modes()[0], AntiAliasingMode::None);
        assert_ne!(anti_aliasing.supported_modes()[1], AntiAliasingMode::None);
    }

    fn has_not_supported_modes(anti_aliasing: &AntiAliasing) {
        assert_eq!(anti_aliasing.supported_modes(), [AntiAliasingMode::None]);
    }
);

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(texture_target(0, Size::new(30, 20), true))
        .child_entity(rectangle())
}

fn rectangle() -> impl BuiltEntity {
    instance_2d(TEXTURE_CAMERAS_2D.get(0), None)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .updated(|t: &mut Transform2D| t.rotation = FRAC_PI_8)
        .updated(|m: &mut Material| m.color = Color::GREEN)
}
