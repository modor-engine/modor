use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, has_pixel_diff, is_same};
use modor_graphics::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Material::new(MATERIAL))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_white"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_color() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Material::new(MATERIAL).with_color(Color::GREEN))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_green"))
        .with_update::<(), _>(|m: &mut Material| m.color = Color::RED)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_red"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_texture() {
    let missing_texture_key = ResKey::new("missing");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Material::new(MATERIAL).with_texture_key(OPAQUE_TEXTURE))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#opaque_texture"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = Some(TRANSPARENT_TEXTURE))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#transparent_texture", 1))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_white"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = Some(missing_texture_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_color_and_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            Material::new(MATERIAL)
                .with_texture_key(OPAQUE_TEXTURE)
                .with_color(Color::RED),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#opaque_texture_red"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = Some(TRANSPARENT_TEXTURE))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#transparent_texture_red", 1))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_red"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_cropped_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            Material::new(MATERIAL)
                .with_texture_key(OPAQUE_TEXTURE)
                .with_texture_position(Vec2::new(0.5, 0.))
                .with_texture_size(Vec2::new(0.5, 1.)),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#cropped_texture", 1))
        .with_update::<(), _>(|m: &mut Material| m.texture_position = Vec2::ZERO)
        .with_update::<(), _>(|m: &mut Material| m.texture_size = Vec2::ONE)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#opaque_texture", 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_front_texture() {
    let missing_texture_key = ResKey::new("missing");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Material::new(MATERIAL).with_front_texture_key(OPAQUE_TEXTURE))
        .updated()
        .with_update::<(), _>(|m: &mut Material| m.front_texture_key = Some(TRANSPARENT_TEXTURE))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture", 10))
        .with_update::<(), _>(|m: &mut Material| m.front_texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#color_white", 10))
        .with_update::<(), _>(|m: &mut Material| m.front_texture_key = Some(missing_texture_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_front_color_and_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            Material::new(MATERIAL)
                .with_front_texture_key(TRANSPARENT_TEXTURE)
                .with_front_color(Color::RED)
                .with_color(Color::GREEN),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture_red", 10))
        .with_update::<(), _>(|m: &mut Material| m.front_color = Color::BLUE)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture_blue", 10));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_ellipse() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            Material::ellipse(MATERIAL)
                .with_color(Color::GREEN)
                .with_texture_key(OPAQUE_TEXTURE)
                .with_front_color(Color::RED)
                .with_front_texture_key(TRANSPARENT_TEXTURE)
                .with_texture_position(Vec2::new(0.5, 0.))
                .with_texture_size(Vec2::new(0.5, 1.)),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#ellipse", 10));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_entity() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Material::new(MATERIAL).with_color(Color::GREEN))
        .updated()
        .with_deleted_entities::<With<Material>>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(target())
        .with_child(opaque_texture())
        .with_child(transparent_texture())
        .with_child(rectangle())
}

fn target() -> impl BuiltEntity {
    let target_key = ResKey::unique("main");
    let texture_key = ResKey::unique("target");
    EntityBuilder::new()
        .with(RenderTarget::new(target_key).with_background_color(Color::DARK_GRAY))
        .with(Texture::from_size(texture_key, Size::new(30, 20)))
        .with(TextureBuffer::default())
        .with_child(Camera2D::new(CAMERA, target_key))
}

fn rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.8, 0.5)))
        .with(Model::rectangle(MATERIAL, CAMERA))
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
    Texture::from_buffer(OPAQUE_TEXTURE, Size::new(4, 4), texture).with_smooth(false)
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
    Texture::from_buffer(TRANSPARENT_TEXTURE, Size::new(4, 4), texture).with_smooth(false)
}

const OPAQUE_TEXTURE: ResKey<Texture> = ResKey::new("opaque");
const TRANSPARENT_TEXTURE: ResKey<Texture> = ResKey::new("transparent");
const MATERIAL: ResKey<Material> = ResKey::new("main");
const CAMERA: ResKey<Camera2D> = ResKey::new("main");
