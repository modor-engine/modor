use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{
    Color, DefaultMaterial2D, Mat, Model2D, Size, Texture, TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;
use std::f32::consts::FRAC_PI_4;

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    assert_same(&mut app, &target, "model#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn delete_model() {
    let (mut app, target) = configure_app();
    root(&mut app).models.clear();
    app.update();
    app.update();
    assert_same(&mut app, &target, "model#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_position() {
    let (mut app, target) = configure_app();
    root(&mut app).models[0].position = Vec2::new(-0.5, 0.5);
    app.update();
    app.update();
    assert_same(&mut app, &target, "model#moved");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target) = configure_app();
    root(&mut app).models[0].size = Vec2::new(0.5, 0.75);
    app.update();
    app.update();
    assert_same(&mut app, &target, "model#scaled");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_rotation() {
    let (mut app, target) = configure_app();
    root(&mut app).models[0].rotation = FRAC_PI_4;
    app.update();
    app.update();
    assert_same(&mut app, &target, "model#rotated");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_material() {
    let (mut app, target) = configure_app();
    let material = root(&mut app).material2.glob();
    root(&mut app).models[0].material = material;
    app.update(); // TODO: remove delay in all tests (sync all buffers at each target update)
    app.update();
    assert_same(&mut app, &target, "model#other_material");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_camera() {
    let (mut app, target) = configure_app();
    let camera = root(&mut app).target2.camera.glob().clone();
    root(&mut app).models[0].camera = camera;
    app.update();
    assert_same(&mut app, &target, "model#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_z_index() {
    let (mut app, target) = configure_app();
    let camera = root(&mut app).target1.camera.glob().clone();
    let material1 = root(&mut app).material1.glob();
    let material2 = root(&mut app).material2.glob();
    let model2 = Model2D::new(&mut app.ctx(), material2.clone());
    let model3 = Model2D::new(&mut app.ctx(), material1);
    let model4 = Model2D::new(&mut app.ctx(), material2);
    root(&mut app).models.extend([model2, model3, model4]);
    root(&mut app).material1.color = Color::BLUE.with_alpha(0.5);
    root(&mut app).material2.color = Color::GREEN;
    root(&mut app).models[0].camera = camera.clone();
    root(&mut app).models[0].z_index = -2;
    root(&mut app).models[0].position = Vec2::ONE * -0.15;
    root(&mut app).models[0].size = Vec2::ONE * 0.25;
    root(&mut app).models[1].camera = camera.clone();
    root(&mut app).models[1].z_index = -1;
    root(&mut app).models[1].position = Vec2::ONE * -0.05;
    root(&mut app).models[1].size = Vec2::ONE * 0.25;
    root(&mut app).models[2].camera = camera.clone();
    root(&mut app).models[2].z_index = 0;
    root(&mut app).models[2].position = Vec2::ONE * 0.05;
    root(&mut app).models[2].size = Vec2::ONE * 0.25;
    root(&mut app).models[3].camera = camera;
    root(&mut app).models[3].z_index = 1;
    root(&mut app).models[3].position = Vec2::ONE * 0.15;
    root(&mut app).models[3].size = Vec2::ONE * 0.25;
    app.update();
    app.update();
    assert_max_component_diff(&mut app, &target, "model#z_index", 10, 1);
    root(&mut app).models[0].z_index = 2;
    root(&mut app).models[1].z_index = 1;
    root(&mut app).models[2].z_index = 0;
    root(&mut app).models[3].z_index = -1;
    app.update();
    app.update();
    assert_max_component_diff(&mut app, &target, "model#reversed_z_index", 10, 1);
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    Root::wait_resources(&mut app);
    let target = root(&mut app).target1.glob().clone();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    material1: Mat<DefaultMaterial2D>,
    material2: Mat<DefaultMaterial2D>,
    models: Vec<Model2D<DefaultMaterial2D>>,
    target1: Res<Texture>,
    target2: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut target1 =
            Res::<Texture>::from_source(ctx, "target1", TextureSource::Size(Size::new(30, 20)));
        target1.is_target_enabled = true;
        target1.is_buffer_enabled = true;
        let mut target2 =
            Res::<Texture>::from_source(ctx, "target2", TextureSource::Size(Size::new(30, 20)));
        target2.is_target_enabled = true;
        target2.is_buffer_enabled = true;
        let material1_data = DefaultMaterial2D::new(ctx);
        let material1 = Mat::new(ctx, "material1", material1_data);
        let mut material2_data = DefaultMaterial2D::new(ctx);
        material2_data.color = Color::RED;
        let material2 = Mat::new(ctx, "material2", material2_data);
        let mut model = Model2D::new(ctx, material1.glob());
        model.camera = target1.camera.glob().clone();
        Self {
            material1,
            material2,
            models: vec![model],
            target1,
            target2,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target1);
        wait_resource(app, |r: &Self| &r.target2);
        app.update();
    }
}
