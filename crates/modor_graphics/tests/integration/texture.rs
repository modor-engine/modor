use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{
    Color, DefaultMaterial2D, Mat, Model2D, Size, Texture, TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::{Res, ResourceState};

const TEXTURE_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/opaque-texture.png"
));

#[modor::test(disabled(macos, android, wasm))]
fn load_from_size() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Size(Size::new(40, 20));
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert_same(&mut app, &glob, "texture#from_size");
    assert_eq!(glob.get(&app.ctx()).size, Size::new(40, 20));
}

#[modor::test(disabled(macos, android, wasm))]
fn load_from_zero_size() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Size(Size::ZERO);
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Loaded
    ));
    assert_same(&mut app, &glob, "texture#empty");
    assert_eq!(glob.get(&app.ctx()).size, Size::ONE);
}

#[modor::test(disabled(macos, android, wasm))]
fn load_from_buffer() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Buffer(
        Size::new(3, 1),
        vec![255, 255, 255, 255, 0, 0, 0, 255, 255, 255, 255, 255],
    );
    root(&mut app).texture.reload_with_source(source);
    app.update();
    assert_same(&mut app, &glob, "texture#from_buffer");
    assert_eq!(glob.get(&app.ctx()).size, Size::new(3, 1));
}

#[modor::test(disabled(macos, android, wasm))]
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

#[modor::test(disabled(macos, android, wasm))]
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

#[modor::test(disabled(macos, android, wasm))]
fn load_from_bytes() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    Root::wait_resources(&mut app);
    assert_same(&mut app, &glob, "texture#from_file");
    assert_eq!(glob.get(&app.ctx()).size, Size::new(4, 4));
}

#[modor::test(disabled(macos, android, wasm))]
fn load_from_path() {
    let (mut app, glob, _) = configure_app();
    let path = "../tests/assets/opaque-texture.png";
    root(&mut app).texture.reload_with_path(path);
    Root::wait_resources(&mut app);
    assert_same(&mut app, &glob, "texture#from_file");
    assert_eq!(glob.get(&app.ctx()).size, Size::new(4, 4));
}

#[modor::test(disabled(macos, android, wasm))]
fn load_file_with_invalid_format() {
    let (mut app, _, _) = configure_app();
    let path = "../tests/assets/text.txt";
    root(&mut app).texture.reload_with_path(path);
    Root::wait_resources(&mut app);
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(macos, android, wasm))]
fn load_corrupted_file() {
    let (mut app, _, _) = configure_app();
    let path = "../tests/assets/corrupted.png";
    root(&mut app).texture.reload_with_path(path);
    Root::wait_resources(&mut app);
    assert!(matches!(
        root(&mut app).texture.state(),
        ResourceState::Error(_)
    ));
}

#[modor::test(disabled(macos, android, wasm))]
fn retrieve_buffer() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    Root::wait_resources(&mut app);
    let ctx = app.ctx();
    let buffer = glob.get(&ctx).buffer(&ctx);
    assert_eq!(buffer.len(), 4 * 4 * 4);
    assert_eq!(buffer[0..4], [255, 0, 0, 255]);
    root(&mut app).texture.is_buffer_enabled = false;
}

#[modor::test(disabled(macos, android, wasm))]
fn retrieve_buffer_when_disabled() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    root(&mut app).texture.is_buffer_enabled = false;
    Root::wait_resources(&mut app);
    let ctx = app.ctx();
    let buffer = glob.get(&ctx).buffer(&ctx);
    assert_eq!(buffer.len(), 0);
}

#[modor::test(disabled(macos, android, wasm))]
fn retrieve_color() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    Root::wait_resources(&mut app);
    let ctx = app.ctx();
    assert_eq!(glob.get(&ctx).color(&ctx, 0, 0), Some(Color::RED));
    assert_eq!(glob.get(&ctx).color(&ctx, 3, 0).map(|c| c.r), Some(1.));
    assert!(glob.get(&ctx).color(&ctx, 3, 0).map(|c| c.g) > Some(0.9));
    assert_eq!(glob.get(&ctx).color(&ctx, 3, 0).map(|c| c.b), Some(0.));
    assert_eq!(glob.get(&ctx).color(&ctx, 0, 3).map(|c| c.r), Some(0.));
    assert_eq!(glob.get(&ctx).color(&ctx, 0, 3).map(|c| c.g), Some(1.));
    assert!(glob.get(&ctx).color(&ctx, 0, 3).map(|c| c.b) < Some(0.1));
    assert_eq!(glob.get(&ctx).color(&ctx, 3, 3).map(|c| c.r), Some(0.));
    assert!(glob.get(&ctx).color(&ctx, 3, 3).map(|c| c.g) < Some(0.1));
    assert_eq!(glob.get(&ctx).color(&ctx, 3, 3).map(|c| c.b), Some(1.));
    assert_eq!(glob.get(&ctx).color(&ctx, 4, 4), None);
}

#[modor::test(disabled(macos, android, wasm))]
fn retrieve_color_when_buffer_disabled() {
    let (mut app, glob, _) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    root(&mut app).texture.is_buffer_enabled = false;
    Root::wait_resources(&mut app);
    let ctx = app.ctx();
    assert_eq!(glob.get(&ctx).color(&ctx, 0, 0), None);
}

#[modor::test(disabled(macos, android, wasm))]
fn set_smooth() {
    let (mut app, _glob, target) = configure_app();
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    Root::wait_resources(&mut app);
    assert_max_component_diff(&mut app, &target, "texture#smooth", 10, 1);
    root(&mut app).texture.is_smooth = false;
    app.update();
    assert_same(&mut app, &target, "texture#not_smooth");
}

#[modor::test(disabled(macos, android, wasm))]
fn set_repeated() {
    let (mut app, _glob, target) = configure_app();
    root(&mut app).material.texture_size = Vec2::ONE * 2.;
    root(&mut app).texture.is_smooth = false;
    let source = TextureSource::Bytes(TEXTURE_BYTES);
    root(&mut app).texture.reload_with_source(source);
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "texture#not_repeated");
    root(&mut app).texture.is_repeated = true;
    app.update();
    assert_same(&mut app, &target, "texture#repeated");
}

fn configure_app() -> (App, GlobRef<TextureGlob>, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let texture = root(&mut app).texture.glob().clone();
    let target = app.get_mut::<Root>().target.glob().clone();
    (app, texture, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    texture: Res<Texture>,
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut target =
            Res::<Texture>::from_source(ctx, "target", TextureSource::Size(Size::new(20, 20)));
        target.is_target_enabled = true;
        target.is_buffer_enabled = true;
        let mut texture = Res::<Texture>::from_source(ctx, "main", TextureSource::Size(Size::ONE));
        texture.is_buffer_enabled = true;
        let mut material_data = DefaultMaterial2D::new(ctx);
        material_data.texture = texture.glob().clone();
        let material = Mat::new(ctx, "main", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.camera = target.camera.glob().clone();
        Self {
            texture,
            material,
            model,
            target,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.texture);
        wait_resource(app, |r: &Self| &r.target);
        app.update();
    }
}
