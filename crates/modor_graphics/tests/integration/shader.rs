use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, ShaderSource, Size, Texture,
    TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;

const SIMPLE_SHADER_PATH: &str = "../tests/assets/simple.wgsl";
const INVALID_SHADER_PATH: &str = "../tests/assets/invalid.wgsl";

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_path() {
    let (mut app, target) = configure_app();
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "shader#default");
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
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "shader#red");
    assert!(!root(&mut app).shader.is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_invalid_code() {
    let (mut app, target) = configure_app();
    Root::wait_resources(&mut app);
    root(&mut app).shader.reload_with_path(INVALID_SHADER_PATH);
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "shader#default");
    assert!(root(&mut app).shader.is_invalid());
    root(&mut app).shader.reload_with_path(SIMPLE_SHADER_PATH);
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "shader#default");
    assert!(!root(&mut app).shader.is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_alpha_replaced() {
    let (mut app, target) = configure_app();
    Root::wait_resources(&mut app);
    root(&mut app).shader.is_alpha_replaced = true;
    app.update();
    assert_same(&mut app, &target, "shader#empty"); // because shader updated after material
    app.update();
    assert_same(&mut app, &target, "shader#not_replaced_alpha");
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.glob().clone();
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
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut target =
            Res::<Texture>::from_source(ctx, "main", TextureSource::Size(Size::new(30, 20)));
        target.is_target_enabled = true;
        target.is_buffer_enabled = true;
        let shader = Res::<Shader<_>>::from_path(ctx, "main", SIMPLE_SHADER_PATH);
        let material_data = TestMaterial {
            color: Color::RED.with_alpha(0.25),
            shader: shader.glob(),
        };
        let material = Mat::new(ctx, "main", material_data);
        let mut model1 = Model2D::new(ctx, material.glob());
        model1.position = Vec2::ZERO;
        model1.size = Vec2::ONE * 0.5;
        model1.camera = target.camera.glob().clone();
        let mut model2 = Model2D::new(ctx, material.glob());
        model1.position = Vec2::ONE * 0.25;
        model2.size = Vec2::ONE * 0.5;
        model2.camera = target.camera.glob().clone();
        model2.z_index = -1;
        Self {
            material,
            shader,
            model1,
            model2,
            target,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target);
        wait_resource(app, |r: &Self| &r.shader);
        app.update();
    }
}

#[derive(Node, Visit)]
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

    fn instance_data(_ctx: &mut Context<'_>, _model: &GlobRef<Model2DGlob>) -> Self::InstanceData {}
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData {
    color: [f32; 4],
}
