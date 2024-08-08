use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State, Updater};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{Color, Size, Sprite2D, Texture, TextureSource};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResourceState};

const TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/opaque-texture.png"
));

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_size() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Size(Size::new(40, 20)))
        .apply(&mut app);
    app.update();
    assert_same(&app, &glob, "texture#from_size");
    assert_eq!(glob.get(&app).glob.size, Size::new(40, 20));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_zero_size() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Size(Size::ZERO))
        .apply(&mut app);
    app.update();
    assert!(matches!(
        root(&mut app).texture.to_ref().get(&app).state(),
        ResourceState::Loaded
    ));
    assert_same(&app, &glob, "texture#empty");
    assert_eq!(glob.get(&app).glob.size, Size::ONE);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_buffer() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Buffer(
            Size::new(3, 1),
            vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255],
        ))
        .apply(&mut app);
    app.update();
    assert_same(&app, &glob, "texture#from_buffer");
    assert_eq!(glob.get(&app).glob.size, Size::new(3, 1));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_empty_buffer() {
    let (mut app, _, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Buffer(Size::ZERO, vec![]))
        .apply(&mut app);
    app.update();
    assert!(matches!(
        root(&mut app).texture.to_ref().get(&app).state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_too_small_buffer() {
    let (mut app, _, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Buffer(Size::ONE, vec![]))
        .apply(&mut app);
    app.update();
    assert!(matches!(
        root(&mut app).texture.to_ref().get(&app).state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_bytes() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &glob, "texture#from_file");
    assert_eq!(glob.get(&app).glob.size, Size::new(4, 4));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_path() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .path("../tests/assets/opaque-texture.png")
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &glob, "texture#from_file");
    assert_eq!(glob.get(&app).glob.size, Size::new(4, 4));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_file_with_invalid_format() {
    let (mut app, _, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .path("../tests/assets/text.txt")
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert!(matches!(
        root(&mut app).texture.to_ref().get(&app).state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_corrupted_file() {
    let (mut app, _, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .path("../tests/assets/corrupted.png")
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert!(matches!(
        root(&mut app).texture.to_ref().get(&app).state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_buffer() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    let buffer = glob.get(&app).glob.buffer(&app);
    assert_eq!(buffer.len(), 4 * 4 * 4);
    assert_eq!(buffer[0..4], [255, 0, 0, 255]);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_buffer_when_disabled() {
    let (mut app, glob, _) = configure_app();
    wait_resources(&mut app);
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .for_inner(&mut app, |inner, app| {
            inner.updater().is_buffer_enabled(false).apply(app)
        })
        .apply(&mut app);
    app.update();
    let buffer = glob.get(&app).glob.buffer(&app);
    assert_eq!(buffer.len(), 0);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_color() {
    let (mut app, glob, _) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_eq!(glob.get(&app).glob.color(&app, 0, 0), Some(Color::RED));
    assert_eq!(glob.get(&app).glob.color(&app, 3, 0).map(|c| c.r), Some(1.));
    assert!(glob.get(&app).glob.color(&app, 3, 0).map(|c| c.g) > Some(0.9));
    assert_eq!(glob.get(&app).glob.color(&app, 3, 0).map(|c| c.b), Some(0.));
    assert_eq!(glob.get(&app).glob.color(&app, 0, 3).map(|c| c.r), Some(0.));
    assert_eq!(glob.get(&app).glob.color(&app, 0, 3).map(|c| c.g), Some(1.));
    assert!(glob.get(&app).glob.color(&app, 0, 3).map(|c| c.b) < Some(0.1));
    assert_eq!(glob.get(&app).glob.color(&app, 3, 3).map(|c| c.r), Some(0.));
    assert!(glob.get(&app).glob.color(&app, 3, 3).map(|c| c.g) < Some(0.1));
    assert_eq!(glob.get(&app).glob.color(&app, 3, 3).map(|c| c.b), Some(1.));
    assert_eq!(glob.get(&app).glob.color(&app, 4, 4), None);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_color_when_buffer_disabled() {
    let (mut app, glob, _) = configure_app();
    wait_resources(&mut app);
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .for_inner(&mut app, |inner, app| {
            inner.updater().is_buffer_enabled(false).apply(app)
        })
        .apply(&mut app);
    app.update();
    assert_eq!(glob.get(&app).glob.color(&app, 0, 0), None);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_smooth() {
    let (mut app, _glob, target) = configure_app();
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "texture#smooth", 10, 1);
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .for_inner(&mut app, |inner, app| {
            inner.updater().is_smooth(false).apply(app)
        })
        .apply(&mut app);
    app.update();
    app.update();
    assert_same(&app, &target, "texture#not_smooth");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_repeated() {
    let (mut app, _glob, target) = configure_app();
    root(&mut app).sprite.material.texture_size = Vec2::ONE * 2.;
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .for_inner(&mut app, |inner, app| {
            inner.updater().is_smooth(false).apply(app)
        })
        .apply(&mut app);
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .source(TextureSource::Bytes(TEXTURE_BYTES))
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    app.update();
    assert_same(&app, &target, "texture#not_repeated");
    root(&mut app)
        .texture
        .to_ref()
        .updater()
        .for_inner(&mut app, |inner, app| {
            inner.updater().is_repeated(true).apply(app)
        })
        .apply(&mut app);
    app.update();
    app.update();
    assert_same(&app, &target, "texture#repeated");
}

fn configure_app() -> (App, GlobRef<Res<Texture>>, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    let texture = root(&mut app).texture.to_ref();
    let target = app.get_mut::<Root>().target.to_ref();
    (app, texture, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    texture: Glob<Res<Texture>>,
    sprite: Sprite2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            texture: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            target: Glob::from_app(app),
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.texture
            .updater()
            .source(TextureSource::Size(Size::ONE))
            .for_inner(app, |inner, app| {
                inner.updater().is_buffer_enabled(true).apply(app)
            })
            .apply(app);
        self.sprite.model.camera = self.target.get(app).camera.glob().to_ref();
        self.sprite.material.texture = self.texture.to_ref();
        self.target
            .updater()
            .source(TextureSource::Size(Size::new(20, 20)))
            .for_inner(app, |inner, app| {
                inner
                    .updater()
                    .is_target_enabled(true)
                    .is_buffer_enabled(true)
                    .apply(app)
            })
            .apply(app);
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}
