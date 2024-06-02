use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{DefaultMaterial2D, Mat, Model2D, Size, Texture, TextureGlob, TextureSource};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_with_one_target() {
    let (mut app, target, other_target) = configure_app();
    assert_same(&mut app, &target, "camera#default");
    assert_same(&mut app, &other_target, "camera#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn remove_target() {
    let (mut app, target, other_target) = configure_app();
    root(&mut app).target.camera.targets.clear();
    app.update();
    assert_same(&mut app, &target, "camera#empty");
    assert_same(&mut app, &other_target, "camera#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn add_target() {
    let (mut app, target, other_target) = configure_app();
    let other_target_glob = root(&mut app).other_target.target.glob().clone();
    root(&mut app).target.camera.targets.push(other_target_glob);
    app.update();
    assert_same(&mut app, &target, "camera#default");
    assert_same(&mut app, &other_target, "camera#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_position() {
    let (mut app, target, _) = configure_app();
    root(&mut app).target.camera.position = Vec2::new(-0.5, 0.5);
    app.update();
    assert_same(&mut app, &target, "camera#moved");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target, _) = configure_app();
    root(&mut app).target.camera.size = Vec2::new(2., 1.5);
    app.update();
    assert_same(&mut app, &target, "camera#scaled");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_rotation() {
    let (mut app, target, _) = configure_app();
    root(&mut app).target.camera.rotation = FRAC_PI_4;
    app.update();
    assert_same(&mut app, &target, "camera#rotated");
}

fn configure_app() -> (App, GlobRef<TextureGlob>, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    Root::wait_resources(&mut app);
    let target = root(&mut app).target.glob().clone();
    let other_target = root(&mut app).other_target.glob().clone();
    (app, target, other_target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
    target: Res<Texture>,
    other_target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut target =
            Res::<Texture>::from_source(ctx, "target1", TextureSource::Size(Size::new(30, 20)));
        target.is_target_enabled = true;
        target.is_buffer_enabled = true;
        let mut other_target =
            Res::<Texture>::from_source(ctx, "target2", TextureSource::Size(Size::new(30, 20)));
        other_target.is_target_enabled = true;
        other_target.is_buffer_enabled = true;
        let material_data = DefaultMaterial2D::new(ctx);
        let material = Mat::new(ctx, "main", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.camera = target.camera.glob().clone();
        Self {
            material,
            model,
            target,
            other_target,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target);
        wait_resource(app, |r: &Self| &r.other_target);
        app.update();
    }
}
