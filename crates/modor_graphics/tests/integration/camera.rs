use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{Camera2D, Size, Sprite2D, Target, Texture, TextureSource, TextureUpdater};
use modor_input::modor_math::Vec2;
use modor_internal::assert_approx_eq;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_with_one_target() {
    let (mut app, target, other_target) = configure_app();
    app.update();
    assert_same(&app, &target, "camera#default");
    assert_same(&app, &other_target, "camera#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn remove_target() {
    let (mut app, target, other_target) = configure_app();
    TextureUpdater::default()
        .camera_targets(vec![])
        .apply(&mut app, &target);
    app.update();
    assert_same(&app, &target, "camera#empty");
    assert_same(&app, &other_target, "camera#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn add_target() {
    let (mut app, target, other_target) = configure_app();
    let other_target_glob = other_target_glob(&app, &other_target);
    TextureUpdater::default()
        .for_camera_targets(|c| c.push(other_target_glob))
        .apply(&mut app, &target);
    app.update();
    assert_same(&app, &target, "camera#default");
    assert_same(&app, &other_target, "camera#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_position_size_rotation() {
    let (mut app, target, _) = configure_app();
    TextureUpdater::default()
        .camera_position(Vec2::new(-0.5, 0.5))
        .camera_size(Vec2::new(2., 1.5))
        .camera_rotation(FRAC_PI_4)
        .apply(&mut app, &target);
    app.update();
    assert_same(&app, &target, "camera#transformed");
    let glob = camera(&mut app).glob().to_ref();
    let world_position = glob
        .get(&app)
        .world_position(Size::new(800, 600), Vec2::new(0., 600.));
    assert_approx_eq!(world_position, Vec2::new(-1.973_139, 0.912_478));
}

fn configure_app() -> (App, GlobRef<Res<Texture>>, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target.to_ref();
    let other_target = root(&mut app).other_target.to_ref();
    (app, target, other_target)
}

fn camera(app: &mut App) -> &Camera2D {
    root(app).target.to_ref().get_mut(app).camera()
}

fn other_target_glob(app: &App, other_target: &Glob<Res<Texture>>) -> GlobRef<Target> {
    other_target.get(app).target().to_ref()
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(FromApp)]
struct Root {
    sprite: Sprite2D,
    target: Glob<Res<Texture>>,
    other_target: Glob<Res<Texture>>,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.other_target);
        self.sprite.model.camera = self.target.get(app).camera().glob().to_ref();
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}
