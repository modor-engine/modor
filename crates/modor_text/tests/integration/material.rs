use modor::{App, With};
use modor_graphics::testing::has_pixel_diff;
use modor_graphics::{
    model_2d, texture_target, Material, Model2DMaterial, Size, TextureBuffer, TEXTURE_CAMERAS_2D,
};
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;
use modor_text::{text_material, Font};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(texture_target(0, Size::new(100, 50), true))
        .with_entity(model_2d(
            TEXTURE_CAMERAS_2D.get(0),
            Model2DMaterial::Key(MATERIAL),
        ))
        .with_entity(text_material(MATERIAL, "rendered\ntext", 30.))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#default", 50));
}

const MATERIAL: ResKey<Material> = ResKey::new("text");
