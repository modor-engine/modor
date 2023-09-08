use modor::{App, BuiltEntity, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    model_2d, texture_target, Color, Material, Model2DMaterial, Size, TextureBuffer, ZIndex2D,
    TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;

#[modor_test(disabled(macos, android, wasm))]
fn create_for_opaque() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(opaque_blue_rectangle(-0.09, 0))
        .with_entity(opaque_blue_rectangle(0.03, u16::MAX - 1))
        .with_entity(opaque_green_rectangle(-0.03, 1))
        .with_entity(opaque_green_rectangle(0.09, u16::MAX).component(Marker))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("z_index#opaque"))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("z_index#opaque_reversed"))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("z_index#opaque_reversed"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_for_transparent() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(transparent_blue_rectangle(-0.09, 0))
        .with_entity(transparent_blue_rectangle(0.03, u16::MAX - 1))
        .with_entity(transparent_green_rectangle(-0.03, 1))
        .with_entity(transparent_green_rectangle(0.09, u16::MAX).component(Marker))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent", 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_reversed", 1))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_reversed", 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_for_opaque_and_transparent() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(opaque_blue_rectangle(-0.09, 0))
        .with_entity(opaque_blue_rectangle(0.03, u16::MAX - 1))
        .with_entity(transparent_green_rectangle(-0.03, 1))
        .with_entity(transparent_green_rectangle(0.09, u16::MAX).component(Marker))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_mix", 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_mix_reversed", 1))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_mix_reversed", 1),
        );
}

fn opaque_blue_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index).updated(|m: &mut Material| m.color = Color::BLUE)
}

fn transparent_blue_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index).updated(|m: &mut Material| m.color = Color::BLUE.with_alpha(0.5))
}

fn opaque_green_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index).updated(|m: &mut Material| m.color = Color::GREEN)
}

fn transparent_green_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index).updated(|m: &mut Material| m.color = Color::GREEN.with_alpha(0.5))
}

fn rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    model_2d(TEXTURE_CAMERAS_2D.get(0), Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| *t.position = Vec2::new(position, position))
        .updated(|t: &mut Transform2D| *t.size = Vec2::ONE * 0.3)
        .component(ZIndex2D::from(z_index))
}

#[derive(Component, NoSystem)]
struct Marker;
