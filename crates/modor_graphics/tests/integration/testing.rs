use image::ImageError;
use log::Level;
use modor::{App, FromApp, GlobRef, RootNode};
use modor_graphics::testing::{assert_max_component_diff, assert_max_pixel_diff, assert_same};
use modor_graphics::{Size, Texture, TextureGlob, TextureSource};
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};
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
    load_different_pixels(&mut app);
    wait_resources(&mut app);
    assert_max_component_diff(&app, &texture, "testing#texture", 2, 1);
    assert_max_component_diff(&app, &texture, "testing#texture", 1, 2);
    assert_max_pixel_diff(&app, &texture, "testing#texture", 1);
}

#[should_panic = "texture is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_different_texture_using_zero_diff() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app);
    wait_resources(&mut app);
    assert_same(&app, &texture, "testing#texture");
}

#[should_panic = "texture is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_different_texture_using_component_diff() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app);
    wait_resources(&mut app);
    assert_max_component_diff(&app, &texture, "testing#texture", 1, 1);
}

#[should_panic = "texture is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_different_texture_using_pixel_count_diff() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app);
    wait_resources(&mut app);
    assert_max_pixel_diff(&app, &texture, "testing#texture", 0);
}

#[should_panic = "texture buffer is empty"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_empty_texture() {
    let (mut app, texture) = configure_app();
    root(&mut app).texture.is_buffer_enabled = false;
    app.update();
    assert_same(&app, &texture, "testing#texture");
}

#[should_panic = "texture width is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_texture_with_different_width() {
    let (mut app, texture) = configure_app();
    load_different_width(&mut app);
    app.update();
    assert_same(&app, &texture, "testing#texture");
}

#[should_panic = "texture height is different"]
#[modor::test(disabled(windows, macos, android, wasm))]
fn compare_to_texture_with_different_height() {
    let (mut app, texture) = configure_app();
    load_different_height(&mut app);
    app.update();
    assert_same(&app, &texture, "testing#texture");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn generate_diff_texture() {
    let (mut app, texture) = configure_app();
    load_different_pixels(&mut app);
    app.update();
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        assert_same(&app, &texture, "testing#texture");
    }));
    assert!(result.is_err());
    let expected_diff = load_image_data("tests/expected/testing#texture_diff.png");
    let actual_diff = load_image_data(env::temp_dir().join("diff_testing#texture.png"));
    assert_eq!(expected_diff.ok(), actual_diff.ok());
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let texture = root(&mut app).texture.glob().to_ref();
    (app, texture)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn load_image_data(path: impl AsRef<Path>) -> Result<Vec<u8>, ImageError> {
    Ok(image::open(path)?.to_rgba8().into_raw())
}

fn load_different_pixels(app: &mut App) {
    let mut buffer = load_image_data("tests/assets/opaque-texture.png").unwrap();
    buffer[40] += 2;
    buffer[41] += 2;
    let source = TextureSource::Buffer(Size::new(4, 4), buffer);
    root(app).texture.reload_with_source(source);
}

fn load_different_width(app: &mut App) {
    let buffer = load_image_data("tests/assets/opaque-texture.png").unwrap();
    let source = TextureSource::Buffer(Size::new(3, 4), buffer);
    root(app).texture.reload_with_source(source);
}

fn load_different_height(app: &mut App) {
    let buffer = load_image_data("tests/assets/opaque-texture.png").unwrap();
    let source = TextureSource::Buffer(Size::new(4, 3), buffer);
    root(app).texture.reload_with_source(source);
}

struct Root {
    texture: Res<Texture>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            texture: Texture::new(app)
                .with_is_buffer_enabled(true)
                .load_from_source(app, TextureSource::Bytes(TEXTURE_BYTES)),
        }
    }
}

impl RootNode for Root {
    fn update(&mut self, app: &mut App) {
        self.texture.update(app);
    }
}
