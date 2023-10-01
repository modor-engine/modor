use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, has_pixel_diff, is_same};
use modor_graphics::{
    model_2d, texture_target, Color, Material, Model2DMaterial, RenderTarget, Size, Texture,
    TextureBuffer, TEXTURE_CAMERAS_2D,
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
    let mut material = Material::new(MATERIAL);
    material.color = Color::GREEN;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_green"))
        .with_update::<(), _>(|m: &mut Material| m.color = Color::RED)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_red"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_texture() {
    let missing_texture_key = ResKey::new("missing");
    let mut material = Material::new(MATERIAL);
    material.texture_key = Some(OPAQUE_TEXTURE);
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#opaque_texture"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = Some(TRANSPARENT_TEXTURE))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#transparent_texture", 1, 1))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_white"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = Some(missing_texture_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_color_and_texture() {
    let mut material = Material::new(MATERIAL);
    material.color = Color::RED;
    material.texture_key = Some(OPAQUE_TEXTURE);
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#opaque_texture_red"))
        .with_update::<(), _>(|m: &mut Material| m.texture_key = Some(TRANSPARENT_TEXTURE))
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("material#transparent_texture_red", 1, 1),
        )
        .with_update::<(), _>(|m: &mut Material| m.texture_key = None)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#color_red"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_cropped_texture() {
    let mut material = Material::new(MATERIAL);
    material.texture_key = Some(OPAQUE_TEXTURE);
    material.texture_position = Vec2::new(0.5, 0.);
    material.texture_size = Vec2::new(0.5, 1.);
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#cropped_texture", 1, 1))
        .with_update::<(), _>(|m: &mut Material| m.texture_position = Vec2::ZERO)
        .with_update::<(), _>(|m: &mut Material| m.texture_size = Vec2::ONE)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("material#opaque_texture", 1, 1));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_front_texture() {
    let missing_texture_key = ResKey::new("missing");
    let mut material = Material::new(MATERIAL);
    material.front_texture_key = Some(OPAQUE_TEXTURE);
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
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
    let mut material = Material::new(MATERIAL);
    material.front_texture_key = Some(TRANSPARENT_TEXTURE);
    material.front_color = Color::RED;
    material.color = Color::GREEN;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture_red", 10))
        .with_update::<(), _>(|m: &mut Material| m.front_color = Color::BLUE)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture_blue", 10));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_ellipse() {
    let mut material = Material::ellipse(MATERIAL);
    material.color = Color::GREEN;
    material.texture_key = Some(OPAQUE_TEXTURE);
    material.texture_position = Vec2::new(0.5, 0.);
    material.texture_size = Vec2::new(0.5, 1.);
    material.front_color = Color::RED;
    material.front_texture_key = Some(TRANSPARENT_TEXTURE);
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#ellipse", 10));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_entity() {
    let mut material = Material::new(MATERIAL);
    material.color = Color::GREEN;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(material)
        .updated()
        .with_deleted_entities::<With<Material>>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(
            texture_target(0, Size::new(30, 20), true)
                .updated(|t: &mut RenderTarget| t.background_color = Color::DARK_GRAY),
        )
        .child_component(opaque_texture())
        .child_component(transparent_texture())
        .child_entity(rectangle())
}

fn rectangle() -> impl BuiltEntity {
    model_2d(TEXTURE_CAMERAS_2D.get(0), Model2DMaterial::Key(MATERIAL))
        .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.8, 0.5))
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
    let mut texture = Texture::from_buffer(OPAQUE_TEXTURE, Size::new(4, 4), texture);
    texture.is_smooth = false;
    texture
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
    let mut texture = Texture::from_buffer(TRANSPARENT_TEXTURE, Size::new(4, 4), texture);
    texture.is_smooth = false;
    texture
}

const OPAQUE_TEXTURE: ResKey<Texture> = ResKey::new("opaque");
const TRANSPARENT_TEXTURE: ResKey<Texture> = ResKey::new("transparent");
const MATERIAL: ResKey<Material> = ResKey::new("main");
