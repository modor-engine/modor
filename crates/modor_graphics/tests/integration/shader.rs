use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, MatGlob, MatUpdater, Material, Model2D, Model2DGlob, Shader, ShaderGlob, ShaderSource,
    ShaderUpdater, Size, Texture, TextureSource, TextureUpdater,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};

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
    ShaderUpdater::default()
        .res(ResUpdater::default().source(ShaderSource::String(code.into())))
        .apply(&mut app, &shader_glob);
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
    ShaderUpdater::default()
        .res(ResUpdater::default().path(INVALID_SHADER_PATH))
        .apply(&mut app, &shader_glob);
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "shader#default");
    assert!(shader(&mut app).is_invalid());
    ShaderUpdater::default()
        .res(ResUpdater::default().path(SIMPLE_SHADER_PATH))
        .apply(&mut app, &shader_glob);
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
    ShaderUpdater::default()
        .is_alpha_replaced(true)
        .apply(&mut app, &shader_glob);
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
    material: MatGlob<TestMaterial>,
    shader: ShaderGlob<TestMaterial>,
    model1: Model2D,
    model2: Model2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let material = MatGlob::from_app(app);
        let model1 = Model2D::new(app).with_material(material.to_ref());
        let model2 = Model2D::new(app).with_material(material.to_ref());
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
        ShaderUpdater::default()
            .res(ResUpdater::default().path(SIMPLE_SHADER_PATH))
            .apply(app, &self.shader);
        MatUpdater::default()
            .shader(self.shader.to_ref())
            .apply(app, &self.material);
        self.model1.position = Vec2::ZERO;
        self.model1.size = Vec2::ONE * 0.5;
        self.model1.camera = self.target.get(app).camera().glob().to_ref();
        self.model2.position = Vec2::ONE * 0.25;
        self.model2.size = Vec2::ONE * 0.5;
        self.model2.z_index = -1;
        self.model2.camera = self.target.get(app).camera().glob().to_ref();
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
    }

    fn update(&mut self, app: &mut App) {
        self.model1.update(app);
        self.model2.update(app);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterial {
    color: [f32; 4],
}

impl Default for TestMaterial {
    fn default() -> Self {
        Self {
            color: Color::RED.with_alpha(0.25).into(),
        }
    }
}

impl Material for TestMaterial {
    type InstanceData = ();

    fn init(app: &mut App, glob: &MatGlob<Self>) {
        MatUpdater::default().is_transparent(true).apply(app, glob);
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}
