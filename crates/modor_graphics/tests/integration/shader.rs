use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Glob, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, IntoMat, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, ShaderSource, Size,
    Texture, TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};

const SIMPLE_SHADER_PATH: &str = "../tests/assets/simple.wgsl";
const INVALID_SHADER_PATH: &str = "../tests/assets/invalid.wgsl";

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_path() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(!root(&mut app).shader.is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_string() {
    let (mut app, target) = configure_app();
    let code = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/assets/red.wgsl"
    ));
    root(&mut app)
        .shader
        .reload_with_source(ShaderSource::String(code.into()));
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#red");
    assert!(!root(&mut app).shader.is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_invalid_code() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    root(&mut app).shader.reload_with_path(INVALID_SHADER_PATH);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(root(&mut app).shader.is_invalid());
    root(&mut app).shader.reload_with_path(SIMPLE_SHADER_PATH);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(!root(&mut app).shader.is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_alpha_replaced() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    app.update();
    root(&mut app).shader.is_alpha_replaced = true;
    app.update();
    assert_same(&app, &target, "shader#empty"); // because shader updated after material
    app.update();
    assert_same(&app, &target, "shader#not_replaced_alpha");
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.glob().to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    material: Mat<TestMaterial>,
    shader: Res<Shader<TestMaterial>>,
    model1: Model2D<TestMaterial>,
    model2: Model2D<TestMaterial>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(30, 20)));
        let shader = Shader::new(app).load_from_path(app, SIMPLE_SHADER_PATH);
        let material = TestMaterial::new(&shader).into_mat(app);
        let model1 = Model2D::new(app, material.glob())
            .with_position(Vec2::ZERO)
            .with_size(Vec2::ONE * 0.5)
            .with_camera(target.camera.glob().to_ref());
        let model2 = Model2D::new(app, material.glob())
            .with_position(Vec2::ONE * 0.25)
            .with_size(Vec2::ONE * 0.5)
            .with_camera(target.camera.glob().to_ref())
            .with_z_index(-1);
        Self {
            material,
            shader,
            model1,
            model2,
            target,
        }
    }
}

struct TestMaterial {
    color: Color,
    shader: ShaderGlobRef<Self>,
}

impl Material for TestMaterial {
    type Data = TestMaterialData;
    type InstanceData = ();

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
        vec![]
    }

    fn is_transparent(&self) -> bool {
        true
    }

    fn data(&self) -> Self::Data {
        TestMaterialData {
            color: self.color.into(),
        }
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}

impl TestMaterial {
    fn new(shader: &Res<Shader<Self>>) -> Self {
        Self {
            color: Color::RED.with_alpha(0.25),
            shader: shader.glob(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData {
    color: [f32; 4],
}
