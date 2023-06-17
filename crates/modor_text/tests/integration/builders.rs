use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics_new2::testing::has_pixel_diff;
use modor_graphics_new2::{Camera2D, Color, Model, RenderTarget, Size, Texture, TextureBuffer};
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_text::{Alignment, Font, TextMaterialBuilder};

// TODO: use modor_test everywhere
#[modor_test(disabled(macos, android, wasm))]
fn create_default_text_material() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(target())
        .with_entity(text())
        .with_entity(TextMaterialBuilder::new(MaterialKey, "rendered\ntext", 30.).build())
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
            TextMaterialBuilder::new(MaterialKey, "rendered\ntext", 30.)
                .with_material(|m| m.with_front_color(Color::BLUE))
                .with_text(|t| t.with_alignment(Alignment::Right))
                .with_texture(|t| t.with_repeated(true))
                .build(),
        )
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("builders#text_material_custom", 50));
}

fn target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Texture::from_size(TextureKey, Size::new(100, 50)))
        .with(TextureBuffer::default())
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Model::rectangle(MaterialKey).with_camera_key(CameraKey))
        .with(Transform2D::new())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextureKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MaterialKey;
