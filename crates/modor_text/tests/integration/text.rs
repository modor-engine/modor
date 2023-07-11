use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::is_same;
use modor_graphics::{Size, Texture, TextureBuffer};
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;
use modor_text::{Alignment, Font, Text};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(text(|_| ()))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#default"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_alignment() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(text(|t| t.alignment = Alignment::Left))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#left"))
        .with_update::<With<TextureBuffer>, _>(|t: &mut Text| t.alignment = Alignment::Right)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("text#right"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_font() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(Font::from_path(
            TTF_FONT,
            "../tests/assets/IrishGrover-Regular.ttf",
        ))
        .with_entity(Font::from_path(
            OTF_FONT,
            "../tests/assets/Foglihtenno07.otf",
        ))
        .with_entity(text(|t| t.font_key = TTF_FONT))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#font_ttf"))
        .with_update::<With<TextureBuffer>, _>(|t: &mut Text| t.font_key = OTF_FONT)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("text#font_otf"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_before_font() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(text(|t| t.font_key = TTF_FONT))
        .updated()
        .with_entity(Font::from_path(
            TTF_FONT,
            "../tests/assets/IrishGrover-Regular.ttf",
        ))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#font_ttf"));
}

fn text(updater: impl FnOnce(&mut Text)) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Text::new("text\nto\nrender", 30.))
        .with(updater)
        .component(Texture::from_size(ResKey::unique("text"), Size::ZERO))
        .component(TextureBuffer::default())
}

const TTF_FONT: ResKey<Font> = ResKey::new("ttf");
const OTF_FONT: ResKey<Font> = ResKey::new("otf");
