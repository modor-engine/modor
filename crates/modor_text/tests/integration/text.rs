use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics_new2::testing::is_same;
use modor_graphics_new2::{Size, Texture, TextureBuffer};
use modor_resources::testing::wait_resource_loading;
use modor_resources::IntoResourceKey;
use modor_text::{Alignment, Font, Text};

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(texture().with(Text::new("text\nto\nrender", 30.)))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#default"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_alignment() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(
            texture().with(Text::new("text\nto\nrender", 30.).with_alignment(Alignment::Left)),
        )
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
            FontKey::Ttf,
            "../tests/assets/IrishGrover-Regular.ttf",
        ))
        .with_entity(Font::from_path(
            FontKey::Otf,
            "../tests/assets/Foglihtenno07.otf",
        ))
        .with_entity(texture().with(Text::new("text\nto\nrender", 30.).with_font(FontKey::Ttf)))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#font_ttf"))
        .with_update::<With<TextureBuffer>, _>(|t: &mut Text| {
            t.font_key = FontKey::Otf.into_key();
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("text#font_otf"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_before_font() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(texture().with(Text::new("text\nto\nrender", 30.).with_font(FontKey::Ttf)))
        .updated()
        .with_entity(Font::from_path(
            FontKey::Ttf,
            "../tests/assets/IrishGrover-Regular.ttf",
        ))
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("text#font_ttf"));
}

fn texture() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::from_size(TextureKey, Size::ZERO))
        .with(TextureBuffer::default())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextureKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FontKey {
    Ttf,
    Otf,
}
