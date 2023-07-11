use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::has_pixel_diff;
use modor_graphics::{Camera2D, Material, Model, RenderTarget, Size, Texture, TextureBuffer};
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;
use modor_text::{text_material, Font};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(target())
        .with_entity(text())
        .with_entity(text_material(MATERIAL, "rendered\ntext", 30.))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#default", 50));
}

fn target() -> impl BuiltEntity {
    let target_key = ResKey::unique("main");
    let texture_key = ResKey::unique("target");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Texture::from_size(texture_key, Size::new(100, 50)))
        .component(TextureBuffer::default())
        .component(Camera2D::new(CAMERA, target_key))
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Model::rectangle(MATERIAL, CAMERA))
        .component(Transform2D::new())
}

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
const MATERIAL: ResKey<Material> = ResKey::new("text");
