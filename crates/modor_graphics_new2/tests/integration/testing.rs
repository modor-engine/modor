use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics_new2::testing::{assert_texture, wait_texture_loading, MaxTextureDiff};
use modor_graphics_new2::{testing, Size, Texture, TextureBuffer, TextureSource};
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::{env, fs, panic};

#[modor_test(disabled(wasm))]
fn assert_texture_with_not_existing_expected() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(same_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| {
                panic::catch_unwind(AssertUnwindSafe(|| {
                    testing::assert_texture(b, "testing#new_expected", MaxTextureDiff::Zero);
                }))
                .expect_err("texture assertion has not panicked");
            })
        });
    let expected_diff = load_image_data(EXPECTED_TEXTURE_PATH);
    let actual_path = "tests/expected/testing#new_expected.png";
    let actual_diff = load_image_data(actual_path);
    assert_eq!(expected_diff, actual_diff);
    fs::remove_file(actual_path).unwrap();
}

#[modor_test(disabled(wasm))]
fn assert_texture_with_same_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(same_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| {
                assert_texture(b, "testing#texture", MaxTextureDiff::Zero);
                assert_texture(b, "testing#texture", MaxTextureDiff::Component(0));
                assert_texture(b, "testing#texture", MaxTextureDiff::Component(255));
                assert_texture(b, "testing#texture", MaxTextureDiff::PixelCount(0));
                assert_texture(b, "testing#texture", MaxTextureDiff::PixelCount(100_000));
            })
        });
}

#[modor_test(disabled(wasm))]
fn assert_texture_with_similar_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(different_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| {
                assert_texture(b, "testing#texture", MaxTextureDiff::Component(2));
                assert_texture(b, "testing#texture", MaxTextureDiff::PixelCount(1));
            })
        });
}

#[should_panic = "texture is different"]
#[modor_test(disabled(wasm))]
fn assert_texture_with_different_texture_using_zero_diff() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(different_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| assert_texture(b, "testing#texture", MaxTextureDiff::Zero))
        });
}

#[should_panic = "texture is different"]
#[modor_test(disabled(wasm))]
fn assert_texture_with_different_texture_using_component_diff() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(different_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| assert_texture(b, "testing#texture", MaxTextureDiff::Component(1)))
        });
}

#[should_panic = "texture is different"]
#[modor_test(disabled(wasm))]
fn assert_texture_with_different_texture_using_pixel_count_diff() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(different_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| assert_texture(b, "testing#texture", MaxTextureDiff::PixelCount(0)))
        });
}

#[should_panic = "texture buffer is empty"]
#[modor_test(disabled(wasm))]
fn assert_texture_with_empty_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(same_texture())
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| assert_texture(b, "testing#texture", MaxTextureDiff::Zero))
        });
}

#[should_panic = "texture width is different"]
#[modor_test(disabled(wasm))]
fn assert_texture_with_different_texture_width() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(texture_with_different_width())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| assert_texture(b, "testing#texture", MaxTextureDiff::Zero))
        });
}

#[should_panic = "texture height is different"]
#[modor_test(disabled(wasm))]
fn assert_texture_with_different_texture_height() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(texture_with_different_height())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| assert_texture(b, "testing#texture", MaxTextureDiff::Zero))
        });
}

#[modor_test(disabled(wasm))]
fn assert_texture_with_different_texture_and_generate_diff_texture() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(different_texture())
        .updated_until_all::<With<Texture>, _>(Some(100), wait_texture_loading)
        .assert::<With<TextureBuffer>>(1, |e| {
            e.has(|b| {
                panic::catch_unwind(AssertUnwindSafe(|| {
                    assert_texture(b, "testing#texture", MaxTextureDiff::Zero);
                }))
                .expect_err("texture assertion has not panicked");
            })
        });
    let expected_diff = load_image_data(EXPECTED_TEXTURE_DIFF_PATH);
    let actual_diff = load_image_data(&env::temp_dir().join("diff_testing#texture.png"));
    assert_eq!(expected_diff, actual_diff);
}

const EXPECTED_TEXTURE_PATH: &str = "tests/expected/testing#texture.png";
const EXPECTED_TEXTURE_DIFF_PATH: &str = "tests/expected/testing#texture_diff.png";

fn same_texture() -> impl BuiltEntity {
    texture(TextureSource::File(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/expected/testing#texture.png"
    ))))
}

fn different_texture() -> impl BuiltEntity {
    let mut buffer = load_image_data(EXPECTED_TEXTURE_PATH);
    buffer[40] += 2;
    texture(TextureSource::RgbaBuffer(buffer, Size::new(4, 4)))
}

fn texture_with_different_width() -> impl BuiltEntity {
    let mut buffer = load_image_data(EXPECTED_TEXTURE_PATH);
    buffer.drain(48..);
    texture(TextureSource::RgbaBuffer(buffer, Size::new(3, 4)))
}

fn texture_with_different_height() -> impl BuiltEntity {
    let mut buffer = load_image_data(EXPECTED_TEXTURE_PATH);
    buffer.drain(48..);
    texture(TextureSource::RgbaBuffer(buffer, Size::new(4, 3)))
}

fn texture(source: TextureSource) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::new("TextureKey", source))
        .with(TextureBuffer::default())
}

fn load_image_data(path: impl AsRef<Path>) -> Vec<u8> {
    image::open(path).unwrap().to_rgba8().into_raw()
}
