use log::Level;
use modor::{App, FromApp, GlobRef, RootNode};
use modor_graphics::testing::assert_same;
use modor_graphics::{Color, Size, Sprite2D, Texture, TextureGlob, TextureSource};
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_default_background() {
    let (app, target) = configure_app();
    assert_same(&app, &target, "target#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target) = configure_app();
    let source = TextureSource::Size(Size::new(20, 30));
    root(&mut app).target.reload_with_source(source);
    app.update();
    assert_same(&app, &target, "target#resized");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_background() {
    let (mut app, target) = configure_app();
    root(&mut app).target.target.background_color = Color::RED;
    app.update();
    assert_same(&app, &target, "target#other_background");
    assert_eq!(target.get(&app).size, Size::new(30, 20));
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target.target.glob().to_ref();
    assert_eq!(target.get(&app).size, Size::new(30, 20));
    let target = root(&mut app).target.glob().to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    sprite: Sprite2D,
    target: Res<Texture>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(30, 20)));
        let sprite = Sprite2D::new(app).with_model(|m| m.camera = target.camera.glob().to_ref());
        Self { sprite, target }
    }
}

impl RootNode for Root {
    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
        self.target.update(app);
    }
}
