use log::Level;
use modor::{App, Context, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{AntiAliasingMode, Size, Sprite2D, Texture, TextureSource};
use modor_input::modor_math::Vec2;
use modor_resources::{Res, ResLoad};
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_supported_modes() {
    let mut app = App::new::<Root>(Level::Info);
    let supported_modes = target(&mut app).target.supported_anti_aliasing_modes();
    assert_eq!(supported_modes[0], AntiAliasingMode::None);
    assert!(supported_modes.contains(&AntiAliasingMode::MsaaX4));
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn enable_supported_anti_aliasing() {
    let mut app = App::new::<Root>(Level::Info);
    let target_glob = target(&mut app).glob().clone();
    app.update();
    assert_same(&mut app, &target_glob, "anti_aliasing#disabled");
    target(&mut app).target.anti_aliasing = AntiAliasingMode::MsaaX4;
    app.update();
    app.update();
    assert_same(&mut app, &target_glob, "anti_aliasing#enabled");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn enable_unsupported_anti_aliasing() {
    let mut app = App::new::<Root>(Level::Info);
    let target_glob = target(&mut app).glob().clone();
    let supported_modes = target(&mut app).target.supported_anti_aliasing_modes();
    if supported_modes.contains(&AntiAliasingMode::MsaaX16) {
        return;
    }
    target(&mut app).target.anti_aliasing = AntiAliasingMode::MsaaX16;
    app.update();
    assert_same(&mut app, &target_glob, "anti_aliasing#disabled");
    app.update();
}

fn target(app: &mut App) -> &mut Res<Texture> {
    &mut app.get_mut::<Root>().target
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
            .with_is_smooth(false)
            .load_from_source(TextureSource::Size(Size::new(30, 20)));
        Self {
            sprite: Sprite2D::new(ctx, "sprite")
                .with_model(|m| m.size = Vec2::ONE * 0.5)
                .with_model(|m| m.rotation = FRAC_PI_4)
                .with_model(|m| m.camera = target.camera.glob().clone()),
            target,
        }
    }
}
