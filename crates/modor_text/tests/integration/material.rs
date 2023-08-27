use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::has_pixel_diff;
use modor_graphics::{texture_target, Material, Model, Size, TextureBuffer, TEXTURE_CAMERAS_2D};
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;
use modor_text::{text_material, Font};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(texture_target(0, Size::new(100, 50), true))
        .with_entity(text())
        .with_entity(text_material(MATERIAL, "rendered\ntext", 30.))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#default", 50));
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Model::rectangle(MATERIAL, TEXTURE_CAMERAS_2D.get(0)))
        .component(Transform2D::new())
}

const MATERIAL: ResKey<Material> = ResKey::new("text");
