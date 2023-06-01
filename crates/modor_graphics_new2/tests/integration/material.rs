use crate::assert_exact_texture;
use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics_new2::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer, TextureSource,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::IntoResourceKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Material::new(MaterialKey))
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#color_white"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_color() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Material::new(MaterialKey).with_color(Color::GREEN))
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#color_green"))
        .with_update::<(), _>(|m: &mut Material| m.color = Color::RED)
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#color_red"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Material::new(MaterialKey).with_texture_key(TextureKey::Opaque))
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#opaque_texture"))
        .with_update::<(), _>(|m: &mut Material| {
            m.texture_key = Some(TextureKey::Transparent.into_key());
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#transparent_texture"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#color_white"))
        .with_update::<(), _>(|m: &mut Material| {
            m.texture_key = Some(TextureKey::Missing.into_key());
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_color_and_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(
            Material::new(MaterialKey)
                .with_texture_key(TextureKey::Opaque)
                .with_color(Color::RED),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#opaque_texture_red"))
        .with_update::<(), _>(|m: &mut Material| {
            m.texture_key = Some(TextureKey::Transparent.into_key());
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#transparent_texture_red"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#color_red"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_cropped_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(
            Material::new(MaterialKey)
                .with_texture_key(TextureKey::Opaque)
                .with_texture_position(Vec2::new(0.5, 0.))
                .with_texture_size(Vec2::new(0.5, 1.)),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#cropped_texture"))
        .with_update::<(), _>(|m: &mut Material| m.texture_position = Vec2::ZERO)
        .with_update::<(), _>(|m: &mut Material| m.texture_size = Vec2::ONE)
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#opaque_texture"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_front_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Material::new(MaterialKey).with_front_texture_key(TextureKey::Opaque))
        .updated()
        .with_update::<(), _>(|m: &mut Material| {
            m.front_texture_key = Some(TextureKey::Transparent.into_key());
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#front_texture"))
        .with_update::<(), _>(|m: &mut Material| m.front_texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#color_white"))
        .with_update::<(), _>(|m: &mut Material| {
            m.front_texture_key = Some(TextureKey::Missing.into_key());
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_front_color_and_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(
            Material::new(MaterialKey)
                .with_front_texture_key(TextureKey::Transparent)
                .with_front_color(Color::RED)
                .with_color(Color::GREEN),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#front_texture_red"))
        .with_update::<(), _>(|m: &mut Material| m.front_color = Color::BLUE)
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#front_texture_blue"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_ellipse() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(
            Material::ellipse(MaterialKey)
                .with_color(Color::GREEN)
                .with_texture_key(TextureKey::Opaque)
                .with_front_color(Color::RED)
                .with_front_texture_key(TextureKey::Transparent)
                .with_texture_position(Vec2::new(0.5, 0.))
                .with_texture_size(Vec2::new(0.5, 1.)),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#ellipse"));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_entity() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Material::new(MaterialKey).with_color(Color::GREEN))
        .updated()
        .with_deleted_entities::<With<Material>>()
        .updated()
        .assert::<With<TextureBuffer>>(1, assert_exact_texture("material#empty"));
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(target())
        .with_child(opaque_texture())
        .with_child(transparent_texture())
        .with_child(Camera2D::new(CameraKey).with_target_key(TargetKey))
        .with_child(rectangle())
}

fn target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey).with_background_color(Color::DARK_GRAY))
        .with(Texture::new(
            TextureKey::Target,
            TextureSource::Size(Size::new(30, 20)),
        ))
        .with(TextureBuffer::default())
}

fn rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.8, 0.5)))
        .with(Model::rectangle(MaterialKey).with_camera_key(CameraKey))
}

fn opaque_texture() -> Texture {
    let white_pixel = [255, 255, 255, 255];
    let gray_pixel = [128, 128, 128, 255];
    let texture = [
        [white_pixel, white_pixel, gray_pixel, gray_pixel],
        [white_pixel, white_pixel, gray_pixel, gray_pixel],
        [gray_pixel, gray_pixel, white_pixel, white_pixel],
        [gray_pixel, gray_pixel, white_pixel, white_pixel],
    ]
    .into_iter()
    .flat_map(|l| l.into_iter().flatten())
    .collect();
    Texture::new(
        TextureKey::Opaque,
        TextureSource::Buffer(texture, Size::new(4, 4)),
    )
    .with_smooth(false)
}

fn transparent_texture() -> Texture {
    let border_pixel = [0, 0, 0, 128];
    let center_pixel = [255, 255, 255, 255];
    let texture = [
        [border_pixel, border_pixel, border_pixel, border_pixel],
        [border_pixel, center_pixel, center_pixel, border_pixel],
        [border_pixel, center_pixel, center_pixel, border_pixel],
        [border_pixel, border_pixel, border_pixel, border_pixel],
    ]
    .into_iter()
    .flat_map(|l| l.into_iter().flatten())
    .collect();
    Texture::new(
        TextureKey::Transparent,
        TextureSource::Buffer(texture, Size::new(4, 4)),
    )
    .with_smooth(false)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TextureKey {
    Target,
    Opaque,
    Transparent,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MaterialKey;
