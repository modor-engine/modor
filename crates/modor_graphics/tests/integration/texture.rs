use log::Level;
use modor::{App, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{Color, Size, Sprite2D, Texture, TextureGlob, TextureSource};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad, ResourceState};

const TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/opaque-texture.png"
));

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_size() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Size(Size::new(40, 20));
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert_same(&app, &glob, "texture#from_size");
    assert_eq!(glob.get(&app).size, Size::new(40, 20));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_zero_size() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Size(Size::ZERO);
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Loaded
    ));
    assert_same(&app, &glob, "texture#empty");
    assert_eq!(glob.get(&app).size, Size::ONE);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_buffer() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Buffer(
        Size::new(3, 1),
        vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255],
    );
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert_same(&app, &glob, "texture#from_buffer");
    assert_eq!(glob.get(&app).size, Size::new(3, 1));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_empty_buffer() {
    let (mut app, _, _) = configure_app();
    let source = TextureSource::Buffer(Size::ZERO, vec![]);
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_too_small_buffer() {
    let (mut app, _, _) = configure_app();
    let source = TextureSource::Buffer(Size::ONE, vec![]);
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_bytes() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    assert_same(&app, &glob, "texture#from_file");
    assert_eq!(glob.get(&app).size, Size::new(4, 4));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_path() {
    let (mut app, glob, _) = configure_app();
    let path = "../tests/assets/opaque-texture.png";
    root(&mut app).texture.reload_with_path(path);
    wait_resources(&mut app);
    assert_same(&app, &glob, "texture#from_file");
    assert_eq!(glob.get(&app).size, Size::new(4, 4));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_file_with_invalid_format() {
    let (mut app, _, _) = configure_app();
    let path = "../tests/assets/text.txt";
    root(&mut app).texture.reload_with_path(path);
    wait_resources(&mut app);
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_corrupted_file() {
    let (mut app, _, _) = configure_app();
    let path = "../tests/assets/corrupted.png";
    root(&mut app).texture.reload_with_path(path);
    wait_resources(&mut app);
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_buffer() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    let buffer = glob.get(&app).buffer(&app);
    assert_eq!(buffer.len(), 4 * 4 * 4);
    assert_eq!(buffer[0..4], [255, 0, 0, 255]);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_buffer_when_disabled() {
    let (mut app, glob, _) = configure_app();
    wait_resources(&mut app);
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    root(&mut app).texture.is_buffer_enabled = false;
    app.update();
    let buffer = glob.get(&app).buffer(&app);
    assert_eq!(buffer.len(), 0);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_color() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    assert_eq!(glob.get(&app).color(&app, 0, 0), Some(Color::RED));
    assert_eq!(glob.get(&app).color(&app, 3, 0).map(|c| c.r), Some(1.));
    assert!(glob.get(&app).color(&app, 3, 0).map(|c| c.g) > Some(0.9));
    assert_eq!(glob.get(&app).color(&app, 3, 0).map(|c| c.b), Some(0.));
    assert_eq!(glob.get(&app).color(&app, 0, 3).map(|c| c.r), Some(0.));
    assert_eq!(glob.get(&app).color(&app, 0, 3).map(|c| c.g), Some(1.));
    assert!(glob.get(&app).color(&app, 0, 3).map(|c| c.b) < Some(0.1));
    assert_eq!(glob.get(&app).color(&app, 3, 3).map(|c| c.r), Some(0.));
    assert!(glob.get(&app).color(&app, 3, 3).map(|c| c.g) < Some(0.1));
    assert_eq!(glob.get(&app).color(&app, 3, 3).map(|c| c.b), Some(1.));
    assert_eq!(glob.get(&app).color(&app, 4, 4), None);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_color_when_buffer_disabled() {
    let (mut app, glob, _) = configure_app();
    wait_resources(&mut app);
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    root(&mut app).texture.is_buffer_enabled = false;
    app.update();
    assert_eq!(glob.get(&app).color(&app, 0, 0), None);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_smooth() {
    let (mut app, _glob, target) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    assert_max_component_diff(&app, &target, "texture#smooth", 10, 1);
    root(&mut app).texture.is_smooth = false;
    app.update();
    assert_same(&app, &target, "texture#not_smooth");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_repeated() {
    let (mut app, _glob, target) = configure_app();
    root(&mut app).sprite.material.texture_size = Vec2::ONE * 2.;
    root(&mut app).texture.is_smooth = false;
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    wait_resources(&mut app);
    assert_same(&app, &target, "texture#not_repeated");
    root(&mut app).texture.is_repeated = true;
    app.update();
    assert_same(&app, &target, "texture#repeated");
}

fn configure_app() -> (App, GlobRef<TextureGlob>, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let texture = root(&mut app).texture.glob().to_ref();
    let target = app.get_mut::<Root>().target.glob().to_ref();
    (app, texture, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    texture: Res<Texture>,
    sprite: Sprite2D,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(20, 20)));
        let texture = Texture::new(app)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::ONE));
        let sprite = Sprite2D::new(app)
            .with_model(|m| m.camera = target.camera.glob().to_ref())
            .with_material(|m| m.texture = texture.glob().to_ref());
        Self {
            texture,
            sprite,
            target,
        }
    }
}
