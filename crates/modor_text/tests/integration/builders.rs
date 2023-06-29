use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::has_pixel_diff;
use modor_graphics::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
};
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;
use modor_text::{Alignment, Font, TextMaterialBuilder};

#[modor_test(disabled(macos, android, wasm))]
fn create_default_text_material() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(target())
        .with_entity(text())
        .with_entity(TextMaterialBuilder::new(MATERIAL, "rendered\ntext", 30.).build())
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("builders#text_material_default", 50));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_custom_text_material() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(target())
        .with_entity(text())
        .with_entity(
            TextMaterialBuilder::new(MATERIAL, "rendered\ntext", 30.)
                .with_material(|m| m.with_front_color(Color::BLUE))
                .with_text(|t| t.with_alignment(Alignment::Right))
                .with_texture(|t| t.with_repeated(true))
                .build(),
        )
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("builders#text_material_custom", 50));
}

fn target() -> impl BuiltEntity {
    let target_key = ResKey::unique("main");
    let texture_key = ResKey::unique("target");
    EntityBuilder::new()
        .with(RenderTarget::new(target_key))
        .with(Texture::from_size(texture_key, Size::new(100, 50)))
        .with(TextureBuffer::default())
        .with(Camera2D::new(CAMERA, target_key))
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Model::rectangle(MATERIAL, CAMERA))
        .with(Transform2D::new())
}

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
const MATERIAL: ResKey<Material> = ResKey::new("text");
