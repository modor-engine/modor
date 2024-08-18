use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{AntiAliasingMode, Size, Sprite2D, Texture, TextureSource, TextureUpdater};
use modor_input::modor_math::Vec2;
use modor_resources::{Res, ResUpdater};
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_supported_modes() {
    let mut app = App::new::<Root>(Level::Info);
    let supported_modes = target_glob(&mut app)
        .get(&app)
        .target()
        .supported_anti_aliasing_modes();
    assert_eq!(supported_modes[0], AntiAliasingMode::None);
    assert!(supported_modes.contains(&AntiAliasingMode::MsaaX4));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn enable_supported_anti_aliasing() {
    let mut app = App::new::<Root>(Level::Debug);
    let target = target_glob(&mut app);
    app.update();
    app.update();
    assert_same(&app, &target, "anti_aliasing#disabled");
    TextureUpdater::default()
        .target_anti_aliasing(AntiAliasingMode::MsaaX4)
        .apply(&mut app, &target);
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "anti_aliasing#enabled", 30, 1);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn enable_unsupported_anti_aliasing() {
    let mut app = App::new::<Root>(Level::Info);
    let target = target_glob(&mut app);
    let supported_modes = target.get(&app).target().supported_anti_aliasing_modes();
    if supported_modes.contains(&AntiAliasingMode::MsaaX16) {
        return;
    }
    TextureUpdater::default()
        .target_anti_aliasing(AntiAliasingMode::MsaaX16)
        .apply(&mut app, &target);
    app.update();
    app.update();
    assert_same(&app, &target, "anti_aliasing#disabled");
    app.update();
}

fn target_glob(app: &mut App) -> GlobRef<Res<Texture>> {
    app.get_mut::<Root>().target.to_ref()
}

#[derive(FromApp)]
struct Root {
    sprite: Sprite2D,
    target: Glob<Res<Texture>>,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .is_smooth(false)
            .apply(app, &self.target);
        self.sprite.model.size = Vec2::ONE * 0.5;
        self.sprite.model.rotation = FRAC_PI_4;
        self.sprite.model.camera = self.target.get(app).camera().glob().to_ref();
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}
