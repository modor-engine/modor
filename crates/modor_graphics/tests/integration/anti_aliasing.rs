use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::has_pixel_diff;
use modor_graphics::{
    AntiAliasing, Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_8;

#[modor_test]
fn retrieve_sample_count() {
    assert_eq!(AntiAliasing::None.sample_count(), 1);
    assert_eq!(AntiAliasing::MsaaX2.sample_count(), 2);
    assert_eq!(AntiAliasing::MsaaX4.sample_count(), 4);
    assert_eq!(AntiAliasing::MsaaX8.sample_count(), 8);
}

#[modor_test(disabled(macos, android, wasm))]
fn run_msaa_in_texture() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(AntiAliasing::None)
        .with_entity(resources())
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#none", 12))
        .with_entity(AntiAliasing::MsaaX4)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("anti_aliasing#msaa_x4", 12));
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(target())
        .child_entity(rectangle())
}

fn target() -> impl BuiltEntity {
    let target_key = ResKey::unique("main");
    let texture_key = ResKey::unique("target");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Texture::from_size(texture_key, Size::new(30, 20)))
        .component(TextureBuffer::default())
        .component(Camera2D::new(CAMERA, target_key))
}

fn rectangle() -> impl BuiltEntity {
    let material_key = ResKey::unique("rectangle");
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::ONE * 0.5)
        .with(|t| *t.rotation = FRAC_PI_8)
        .component(Model::rectangle(material_key, CAMERA))
        .component(Material::new(material_key))
        .with(|m| m.color = Color::GREEN)
}

const CAMERA: ResKey<Camera2D> = ResKey::new("default");
