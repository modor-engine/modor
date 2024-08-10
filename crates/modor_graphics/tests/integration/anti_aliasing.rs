use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{AntiAliasingMode, Size, Sprite2D, Texture, TextureSource};
use modor_input::modor_math::Vec2;
use modor_resources::Res;
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_supported_modes() {
    let mut app = App::new::<Root>(Level::Info);
    let supported_modes = target_glob(&mut app)
        .get(&app)
        .target
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
    target
        .updater()
        .inner(|i, _| i.target_anti_aliasing(AntiAliasingMode::MsaaX4))
        .apply(&mut app);
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "anti_aliasing#enabled", 30, 1);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn enable_unsupported_anti_aliasing() {
    let mut app = App::new::<Root>(Level::Info);
    let target = target_glob(&mut app);
    let supported_modes = target.get(&app).target.supported_anti_aliasing_modes();
    if supported_modes.contains(&AntiAliasingMode::MsaaX16) {
        return;
    }
    target
        .updater()
        .inner(|i, _| i.target_anti_aliasing(AntiAliasingMode::MsaaX16))
        .apply(&mut app);
    app.update();
    app.update();
    assert_same(&app, &target, "anti_aliasing#disabled");
    app.update();
}

fn target_glob(app: &mut App) -> GlobRef<Res<Texture>> {
    app.get_mut::<Root>().target.to_ref()
}

struct Root {
    sprite: Sprite2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            sprite: Sprite2D::new(app),
            target: Glob::from_app(app),
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.target
            .updater()
            .source(TextureSource::Size(Size::new(30, 20)))
            .inner(|i, _| i.is_target_enabled(true))
            .inner(|i, _| i.is_buffer_enabled(true))
            .inner(|i, _| i.is_smooth(false))
            .apply(app);
        self.sprite.model.size = Vec2::ONE * 0.5;
        self.sprite.model.rotation = FRAC_PI_4;
        self.sprite.model.camera = self.target.get(app).camera.glob().to_ref();
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}
