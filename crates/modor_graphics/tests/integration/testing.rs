use image::ImageError;
use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::{assert_max_component_diff, assert_max_pixel_diff, assert_same};
use modor_graphics::{Size, Texture, TextureSource, TextureUpdater};
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::{env, fs, panic};

const TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/opaque-texture.png"
));

#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_not_existing_expected() {
    let (mut app, texture) = configure_app();
    wait_resources(&mut app);
    app.update();
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        assert_same(&app, &texture, "testing#temporary");
    }));
    let actual_path = "tests/expected/testing#temporary.png";
    let actual_image = load_image_data(actual_path);
    let expected_image = load_image_data("tests/assets/opaque-texture.png");
    assert!(fs::remove_file(actual_path).is_ok());
    assert!(result.is_err());
    assert_eq!(actual_image.ok(), expected_image.ok());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_same_texture() {
    let (mut app, texture) = configure_app();
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &texture, "testing#texture");
    assert_max_component_diff(&app, &texture, "testing#texture", 0, 1);
    assert_max_component_diff(&app, &texture, "testing#texture", 0, 2);
    assert_max_component_diff(&app, &texture, "testing#texture", 0, 10);
    assert_max_component_diff(&app, &texture, "testing#texture", 255, 1);
    assert_max_pixel_diff(&app, &texture, "testing#texture", 0);
    assert_max_pixel_diff(&app, &texture, "testing#texture", 100_000);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_similar_texture() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app, &texture);
    wait_resources(&mut app);
    assert_max_component_diff(&app, &texture, "testing#texture", 2, 1);
    assert_max_component_diff(&app, &texture, "testing#texture", 1, 2);
    assert_max_pixel_diff(&app, &texture, "testing#texture", 1);
}

#[should_panic = "texture is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_different_texture_using_zero_diff() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app, &texture);
    wait_resources(&mut app);
    assert_same(&app, &texture, "testing#texture");
}

#[should_panic = "texture is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_different_texture_using_component_diff() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app, &texture);
    wait_resources(&mut app);
    assert_max_component_diff(&app, &texture, "testing#texture", 1, 1);
}

#[should_panic = "texture is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_different_texture_using_pixel_count_diff() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app, &texture);
    wait_resources(&mut app);
    assert_max_pixel_diff(&app, &texture, "testing#texture", 0);
}

#[should_panic = "texture buffer is empty"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_empty_texture() {
    let (mut app, texture) = configure_app();
    TextureUpdater::default()
        .is_buffer_enabled(false)
        .apply(&mut app, &texture);
    app.update();
    assert_same(&app, &texture, "testing#texture");
}

#[should_panic = "texture width is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_texture_with_different_width() {
    let (mut app, texture) = configure_app();
    load_different_width(&mut app, &texture);
    app.update();
    assert_same(&app, &texture, "testing#texture");
}

#[should_panic = "texture height is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_texture_with_different_height() {
    let (mut app, texture) = configure_app();
    load_different_height(&mut app, &texture);
    app.update();
    assert_same(&app, &texture, "testing#texture");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn generate_diff_texture() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app, &texture);
    app.update();
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        assert_same(&app, &texture, "testing#texture");
    }));
    assert!(result.is_err());
    let expected_diff = load_image_data("tests/expected/testing#texture_diff.png");
    let actual_diff = load_image_data(env::temp_dir().join("diff_testing#texture.png"));
    assert_eq!(expected_diff.ok(), actual_diff.ok());
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    let texture = root(&mut app).texture.to_ref();
    (app, texture)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn load_image_data(path: impl AsRef<Path>) -> Result<Vec<u8>, ImageError> {
    Ok(image::open(path)?.to_rgba8().into_raw())
}

fn load_different_pixels(app: &mut App, texture: &Glob<Res<Texture>>) {
    let mut buffer = load_image_data("tests/assets/opaque-texture.png").unwrap();
    buffer[40] += 2;
    buffer[41] += 2;
    TextureUpdater::default()
        .res(ResUpdater::default().source(TextureSource::Buffer(Size::new(4, 4), buffer)))
        .apply(app, texture);
}

fn load_different_width(app: &mut App, texture: &Glob<Res<Texture>>) {
    let buffer = load_image_data("tests/assets/opaque-texture.png").unwrap();
    TextureUpdater::default()
        .res(ResUpdater::default().source(TextureSource::Buffer(Size::new(3, 4), buffer)))
        .apply(app, texture);
}

fn load_different_height(app: &mut App, texture: &Glob<Res<Texture>>) {
    let buffer = load_image_data("tests/assets/opaque-texture.png").unwrap();
    TextureUpdater::default()
        .res(ResUpdater::default().source(TextureSource::Buffer(Size::new(4, 3), buffer)))
        .apply(app, texture);
}

#[derive(FromApp)]
struct Root {
    texture: Glob<Res<Texture>>,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Bytes(TEXTURE_BYTES)))
            .is_buffer_enabled(true)
            .apply(app, &self.texture);
    }
}
