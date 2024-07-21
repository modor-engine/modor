use log::Level;
use modor::{App, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{Camera2D, Size, Sprite2D, Texture, TextureGlob, TextureSource};
use modor_input::modor_math::Vec2;
use modor_internal::assert_approx_eq;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_with_one_target() {
    let (app, target, other_target) = configure_app();
    assert_same(&app, &target, "camera#default");
    assert_same(&app, &other_target, "camera#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn remove_target() {
    let (mut app, target, other_target) = configure_app();
    camera(&mut app).targets.clear();
    app.update();
    assert_same(&app, &target, "camera#empty");
    assert_same(&app, &other_target, "camera#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn add_target() {
    let (mut app, target, other_target) = configure_app();
    let other_target_glob = root(&mut app).other_target.target.glob().clone();
    camera(&mut app).targets.push(other_target_glob);
    app.update();
    assert_same(&app, &target, "camera#default");
    assert_same(&app, &other_target, "camera#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_position_size_rotation() {
    let (mut app, target, _) = configure_app();
    let position = Vec2::new(-0.5, 0.5);
    let size = Vec2::new(2., 1.5);
    camera(&mut app).position = position;
    camera(&mut app).size = size;
    camera(&mut app).rotation = FRAC_PI_4;
    app.update();
    assert_same(&app, &target, "camera#transformed");
    let glob = camera(&mut app).glob().clone();
    let world_position = glob
        .get(&app)
        .world_position(Size::new(800, 600), Vec2::new(0., 600.));
    assert_approx_eq!(world_position, Vec2::new(-1.973_139, 0.912_478));
}

fn configure_app() -> (App, GlobRef<TextureGlob>, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target.glob().clone();
    let other_target = root(&mut app).other_target.glob().clone();
    (app, target, other_target)
}

fn camera(app: &mut App) -> &mut Camera2D {
    &mut root(app).target.camera
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    sprite: Sprite2D,
    target: Res<Texture>,
    other_target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(30, 20)));
        let other_target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(30, 20)));
        let sprite = Sprite2D::new(app).with_model(|m| m.camera = target.camera.glob().clone());
        Self {
            sprite,
            target,
            other_target,
        }
    }
}
