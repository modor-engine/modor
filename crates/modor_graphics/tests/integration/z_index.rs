use modor::{App, BuiltEntity, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    instance_2d, texture_target, Color, Default2DMaterial, Size, Texture, TextureBuffer, ZIndex2D,
    TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;

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
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent", 1, 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_reversed", 1, 1))
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_reversed", 1, 1));
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
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_mix", 1, 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_mix_reversed", 1, 1),
        )
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_mix_reversed", 1, 1),
        );
}

#[modor_test(disabled(macos, android, wasm))]
fn create_for_transparent_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(transparent_blue_texture_rectangle(-0.09, 0))
        .with_entity(transparent_blue_texture_rectangle(0.03, u16::MAX - 1))
        .with_entity(transparent_green_texture_rectangle(-0.03, 1))
        .with_entity(transparent_green_texture_rectangle(0.09, u16::MAX).component(Marker))
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_texture", 10, 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_texture_reversed", 10, 1),
        )
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_texture_reversed", 10, 1),
        );
}

#[modor_test(disabled(macos, android, wasm))]
fn create_for_transparent_front_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(transparent_blue_front_texture_rectangle(-0.09, 0))
        .with_entity(transparent_blue_front_texture_rectangle(0.03, u16::MAX - 1))
        .with_entity(transparent_green_front_texture_rectangle(-0.03, 1))
        .with_entity(transparent_green_front_texture_rectangle(0.09, u16::MAX).component(Marker))
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_component_diff("z_index#transparent_texture", 10, 1))
        .with_update::<(), _>(|i: &mut ZIndex2D| *i = ZIndex2D::from(u16::MAX - u16::from(*i)))
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_texture_reversed", 10, 1),
        )
        .with_deleted_components::<With<Marker>, ZIndex2D>()
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("z_index#transparent_texture_reversed", 10, 1),
        );
}

fn opaque_blue_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index).updated(|m: &mut Default2DMaterial| m.color = Color::BLUE)
}

fn transparent_blue_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index)
        .updated(|m: &mut Default2DMaterial| m.color = Color::BLUE.with_alpha(0.5))
}

fn opaque_green_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index).updated(|m: &mut Default2DMaterial| m.color = Color::GREEN)
}

fn transparent_green_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    rectangle(position, z_index)
        .updated(|m: &mut Default2DMaterial| m.color = Color::GREEN.with_alpha(0.5))
}

fn transparent_blue_texture_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    let texture_key = ResKey::unique("transparent-green-texture-rectangle");
    opaque_blue_rectangle(position, z_index)
        .updated(|m: &mut Default2DMaterial| m.texture_key = Some(texture_key))
        .component(Texture::from_path(
            texture_key,
            "../tests/assets/transparent-texture.png",
        ))
}

fn transparent_green_texture_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    let texture_key = ResKey::unique("transparent-green-texture-rectangle");
    opaque_green_rectangle(position, z_index)
        .updated(|m: &mut Default2DMaterial| m.texture_key = Some(texture_key))
        .component(Texture::from_path(
            texture_key,
            "../tests/assets/transparent-texture.png",
        ))
        .with(|t| t.is_smooth = false)
}

fn transparent_blue_front_texture_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    let texture_key = ResKey::unique("transparent-green-texture-rectangle");
    rectangle(position, z_index)
        .updated(|m: &mut Default2DMaterial| m.front_texture_key = Some(texture_key))
        .updated(|m: &mut Default2DMaterial| m.front_color = Color::BLUE)
        .updated(|m: &mut Default2DMaterial| m.color = Color::INVISIBLE)
        .component(Texture::from_path(
            texture_key,
            "../tests/assets/transparent-texture.png",
        ))
}

fn transparent_green_front_texture_rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    let texture_key = ResKey::unique("transparent-green-texture-rectangle");
    rectangle(position, z_index)
        .updated(|m: &mut Default2DMaterial| m.front_texture_key = Some(texture_key))
        .updated(|m: &mut Default2DMaterial| m.front_color = Color::GREEN)
        .updated(|m: &mut Default2DMaterial| m.color = Color::INVISIBLE)
        .component(Texture::from_path(
            texture_key,
            "../tests/assets/transparent-texture.png",
        ))
        .with(|t| t.is_smooth = false)
}

fn rectangle(position: f32, z_index: u16) -> impl BuiltEntity {
    instance_2d::<Default2DMaterial>(TEXTURE_CAMERAS_2D.get(0), None)
        .updated(|t: &mut Transform2D| t.position = Vec2::new(position, position))
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.3)
        .component(ZIndex2D::from(z_index))
}

#[derive(Component, NoSystem)]
struct Marker;
