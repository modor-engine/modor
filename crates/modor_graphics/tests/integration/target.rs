use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State, Updater};
use modor_graphics::testing::assert_same;
use modor_graphics::{Color, Size, Sprite2D, Target, Texture, TextureSource};
use modor_resources::testing::wait_resources;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_default_background() {
    let (mut app, target) = configure_app();
    app.update();
    assert_same(&app, &target, "target#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target) = configure_app();
    target
        .updater()
        .source(TextureSource::Size(Size::new(20, 30)))
        .apply(&mut app);
    app.update();
    assert_same(&app, &target, "target#resized");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_background() {
    let (mut app, target) = configure_app();
    target_ref(&mut app).background_color = Color::RED;
    app.update();
    assert_same(&app, &target, "target#other_background");
    assert_eq!(target.get(&app).glob.size, Size::new(30, 20));
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = target_ref(&mut app).glob().to_ref();
    assert_eq!(target.get(&app).size, Size::new(30, 20));
    let target = root(&mut app).target.to_ref();
    (app, target)
}

fn target_ref(app: &mut App) -> &mut Target {
    &mut root(app).target.to_ref().get_mut(app).target
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    sprite: Sprite2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let sprite = Sprite2D::new(app);
        Self { sprite, target }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.sprite.model.camera = self.target.get(app).camera.glob().to_ref();
        self.target
            .updater()
            .source(TextureSource::Size(Size::new(30, 20)))
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
