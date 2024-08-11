use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{
    Camera2DGlob, Color, DefaultMaterial2D, IntoMat, Mat, Model2D, Size, Texture, TextureSource,
    TextureUpdater,
};
use modor_input::modor_math::Vec2;
use modor_physics::{Body2D, Body2DUpdater};
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    app.update();
    assert_same(&app, &target, "model#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn delete_model() {
    let (mut app, target) = configure_app();
    root(&mut app).models.clear();
    app.update();
    app.update();
    assert_same(&app, &target, "model#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_position() {
    let (mut app, target) = configure_app();
    root(&mut app).models[0].position = Vec2::new(-0.5, 0.5);
    app.update();
    app.update();
    assert_same(&app, &target, "model#moved");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_size() {
    let (mut app, target) = configure_app();
    root(&mut app).models[0].size = Vec2::new(0.5, 0.75);
    app.update();
    app.update();
    assert_same(&app, &target, "model#scaled");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_rotation() {
    let (mut app, target) = configure_app();
    root(&mut app).models[0].rotation = FRAC_PI_4;
    app.update();
    app.update();
    assert_same(&app, &target, "model#rotated");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_body() {
    let (mut app, target) = configure_app();
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .position(Vec2::new(-0.25, -0.25))
        .size(Vec2::new(0.5, 0.25))
        .rotation(FRAC_PI_2)
        .apply(&mut app, &body);
    root(&mut app).models[0].body = Some(body.to_ref());
    app.update();
    app.update();
    assert_same(&app, &target, "model#with_body");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_z_index() {
    let (mut app, target) = configure_app();
    let camera = camera1(&mut app);
    let material1 = root(&mut app).material1.glob();
    let material2 = root(&mut app).material2.glob();
    let model2 = Model2D::new(&mut app, material2.clone());
    let model3 = Model2D::new(&mut app, material1);
    let model4 = Model2D::new(&mut app, material2);
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
    assert_max_component_diff(&app, &target, "model#z_index", 10, 1);
    root(&mut app).models[0].z_index = 2;
    root(&mut app).models[1].z_index = 1;
    root(&mut app).models[2].z_index = 0;
    root(&mut app).models[3].z_index = -1;
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "model#reversed_z_index", 10, 1);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_camera() {
    let (mut app, target) = configure_app();
    let camera = camera2(&mut app);
    root(&mut app).models[0].camera = camera;
    app.update();
    app.update();
    assert_same(&app, &target, "model#empty");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_material() {
    let (mut app, target) = configure_app();
    let material = root(&mut app).material2.glob();
    root(&mut app).models[0].material = material;
    app.update();
    app.update();
    assert_same(&app, &target, "model#other_material");
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target1.to_ref();
    (app, target)
}

fn camera1(app: &mut App) -> GlobRef<Camera2DGlob> {
    root(app).target1.to_ref().get(app).camera.glob().to_ref()
}

fn camera2(app: &mut App) -> GlobRef<Camera2DGlob> {
    root(app).target2.to_ref().get(app).camera.glob().to_ref()
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    material1: Mat<DefaultMaterial2D>,
    material2: Mat<DefaultMaterial2D>,
    models: Vec<Model2D<DefaultMaterial2D>>,
    target1: Glob<Res<Texture>>,
    target2: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target1 = Glob::from_app(app);
        let target2 = Glob::from_app(app);
        let material1 = DefaultMaterial2D::new(app).into_mat(app);
        let material2 = DefaultMaterial2D::new(app).into_mat(app);
        let model = Model2D::new(app, material1.glob());
        Self {
            material1,
            material2,
            models: vec![model],
            target1,
            target2,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.material2.color = Color::RED;
        self.models[0].camera = self.target1.get(app).camera.glob().to_ref();
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target1);
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target2);
    }

    fn update(&mut self, app: &mut App) {
        self.material1.update(app);
        self.material2.update(app);
        for model in &mut self.models {
            model.update(app);
        }
    }
}
