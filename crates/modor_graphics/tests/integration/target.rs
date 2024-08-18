use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{Color, Size, Sprite2D, Target, Texture, TextureSource, TextureUpdater};
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_default_background() {
    let (mut app, target) = configure_app();
    app.update();
    assert_same(&app, &target, "target#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target) = configure_app();
    TextureUpdater::default()
        .res(ResUpdater::default().source(TextureSource::Size(Size::new(20, 30))))
        .apply(&mut app, &target);
    app.update();
    assert_same(&app, &target, "target#resized");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_background() {
    let (mut app, target) = configure_app();
    TextureUpdater::default()
        .target_background_color(Color::RED)
        .apply(&mut app, &target);
    app.update();
    assert_same(&app, &target, "target#other_background");
    assert_eq!(target.get(&app).size(), Size::new(30, 20));
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = target_ref(&mut app).glob().to_ref();
    assert_eq!(target.get(&app).size, Size::new(30, 20));
    let target = root(&mut app).target.to_ref();
    (app, target)
}

fn target_ref(app: &mut App) -> &Target {
    root(app).target.to_ref().get_mut(app).target()
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(FromApp)]
struct Root {
    sprite: Sprite2D,
    target: Glob<Res<Texture>>,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.sprite.model.camera = self.target.get(app).camera().glob().to_ref();
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}
