use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{Color, Size, Sprite2D, Texture, TextureGlob, TextureSource};
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_default_background() {
    let (mut app, target) = configure_app();
    assert_same(&mut app, &target, "target#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target) = configure_app();
    let source = TextureSource::Size(Size::new(20, 30));
    root(&mut app).target.reload_with_source(source);
    app.update();
    assert_same(&mut app, &target, "target#resized");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_background() {
    let (mut app, target) = configure_app();
    root(&mut app).target.target.background_color = Color::RED;
    app.update();
    assert_same(&mut app, &target, "target#other_background");
    assert_eq!(target.get(&app.ctx()).size, Size::new(30, 20));
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target.target.glob().clone();
    assert_eq!(target.get(&app.ctx()).size, Size::new(30, 20));
    let target = root(&mut app).target.glob().clone();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    sprite: Sprite2D,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let target = Texture::new(ctx, "target")
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(ctx, TextureSource::Size(Size::new(30, 20)));
        let sprite =
            Sprite2D::new(ctx, "main").with_model(|m| m.camera = target.camera.glob().clone());
        Self { sprite, target }
    }
}
