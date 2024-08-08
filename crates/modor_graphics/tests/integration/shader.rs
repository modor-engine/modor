use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State, Updater};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, IntoMat, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlob, ShaderGlobRef,
    ShaderSource, Size, Texture, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::Res;

const SIMPLE_SHADER_PATH: &str = "../tests/assets/simple.wgsl";
const INVALID_SHADER_PATH: &str = "../tests/assets/invalid.wgsl";

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_path() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(!shader(&mut app).is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_from_string() {
    let (mut app, target) = configure_app();
    let shader_glob = root(&mut app).shader.to_ref();
    let code = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/assets/red.wgsl"
    ));
    shader_glob
        .updater()
        .source(ShaderSource::String(code.into()))
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#red");
    assert!(!shader(&mut app).is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn load_invalid_code() {
    let (mut app, target) = configure_app();
    let shader_glob = root(&mut app).shader.to_ref();
    wait_resources(&mut app);
    shader_glob
        .updater()
        .path(INVALID_SHADER_PATH)
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(shader(&mut app).is_invalid());
    shader_glob
        .updater()
        .path(SIMPLE_SHADER_PATH)
        .apply(&mut app);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(!shader(&mut app).is_invalid());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_alpha_replaced() {
    let (mut app, target) = configure_app();
    let shader_glob = root(&mut app).shader.to_ref();
    wait_resources(&mut app);
    app.update();
    shader_glob
        .updater()
        .for_inner(&mut app, |inner, app| {
            inner.updater().is_alpha_replaced(true).apply(app)
        })
        .apply(&mut app);
    app.update();
    assert_same(&app, &target, "shader#empty"); // because shader updated after material
    app.update();
    assert_same(&app, &target, "shader#not_replaced_alpha");
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn shader(app: &mut App) -> &Res<Shader> {
    app.get_mut::<Root>().shader.to_ref().get(app)
}

struct Root {
    material: Mat<TestMaterial>,
    shader: ShaderGlob<TestMaterial>,
    model1: Model2D<TestMaterial>,
    model2: Model2D<TestMaterial>,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let material = TestMaterial::new(&shader).into_mat(app);
        let model1 = Model2D::new(app, material.glob());
        let model2 = Model2D::new(app, material.glob());
        Self {
            material,
            shader,
            model1,
            model2,
            target,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.shader.updater().path(SIMPLE_SHADER_PATH).apply(app);
        self.model1.position = Vec2::ZERO;
        self.model1.size = Vec2::ONE * 0.5;
        self.model1.camera = self.target.get(app).camera.glob().to_ref();
        self.model2.position = Vec2::ONE * 0.25;
        self.model2.size = Vec2::ONE * 0.5;
        self.model2.z_index = -1;
        self.model2.camera = self.target.get(app).camera.glob().to_ref();
        self.target
            .updater()
            .source(TextureSource::Size(Size::new(30, 20)))
            .for_inner(app, |inner, app| {
                inner
                    .updater()
                    .is_target_enabled(true)
                    .is_buffer_enabled(true)
                    .apply(app)
            })
            .apply(app);
    }

    fn update(&mut self, app: &mut App) {
        self.material.update(app);
        self.model1.update(app);
        self.model2.update(app);
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

    fn textures(&self) -> Vec<GlobRef<Res<Texture>>> {
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
    fn new(shader: &ShaderGlob<Self>) -> Self {
        Self {
            color: Color::RED.with_alpha(0.25),
            shader: shader.to_ref(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData {
    color: [f32; 4],
}
