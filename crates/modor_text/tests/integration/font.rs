use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::is_same;
use modor_graphics::{Size, Texture, TextureBuffer};
use modor_resources::testing::wait_resource_loading;
use modor_resources::{Resource, ResourceLoadingError, ResourceState};
use modor_text::{Font, FontSource, Text};
use std::thread;
use std::time::Duration;

const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/Foglihtenno07.otf"
));

#[modor_test(disabled(macos, android, wasm))]
fn create_from_path() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(Font::from_path(
            FontKey,
            "../tests/assets/IrishGrover-Regular.ttf",
        ))
        .with_entity(text())
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("font#text_ttf"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_file() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(Font::from_file(FontKey, FONT_DATA))
        .with_entity(text())
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("font#text_otf"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_unsupported_format() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(
            EntityBuilder::new()
                .with(Font::from_path(FontKey, "../tests/assets/text.txt"))
                .with(CustomFont),
        )
        .assert::<With<CustomFont>>(1, |e| {
            e.has(|f: &Font| assert_eq!(f.state(), ResourceState::NotLoaded))
        })
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<CustomFont>>(1, |e| {
            e.has(|f: &Font| {
                assert!(matches!(
                    f.state(),
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn set_source() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(text())
        .with_entity(
            EntityBuilder::new()
                .with(Font::from_path(
                    FontKey,
                    "../tests/assets/IrishGrover-Regular.ttf",
                ))
                .with(CustomFont),
        )
        .updated_until_all::<With<Font>, Font>(Some(100), wait_resource_loading)
        .assert::<With<TextureBuffer>>(1, is_same("font#text_ttf"))
        .with_update::<With<CustomFont>, _>(|f: &mut Font| {
            f.set_source(FontSource::File(FONT_DATA));
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("font#text_ttf"))
        .with_update::<With<TextureBuffer>, _>(|_: &mut Font| thread::sleep(Duration::from_secs(5)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("font#text_otf"));
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::from_size(TextureKey, Size::ZERO))
        .with(TextureBuffer::default())
        .with(Text::new("text", 20.).with_font(FontKey))
}

#[derive(Component, NoSystem)]
struct CustomFont;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextureKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontKey;
