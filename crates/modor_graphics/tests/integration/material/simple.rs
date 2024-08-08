use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State, Updater};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{
    Color, IntoMat, Mat, Material, Model2D, Model2DGlob, ShaderGlob, ShaderGlobRef, Size, Texture,
    TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_textures_less_than_shader() {
    let (mut app, target) = configure_app();
    root(&mut app).material.textures = vec![];
    app.update();
    app.update();
    assert_same(&app, &target, "material#no_texture");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_textures_more_than_shader() {
    let (mut app, target) = configure_app();
    let texture = root(&mut app).texture.to_ref();
    root(&mut app).material.textures = vec![texture.clone(), texture];
    app.update();
    assert_same(&app, &target, "material#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_color_opaque() {
    let (mut app, target) = configure_app();
    root(&mut app).material.color = Color::WHITE;
    app.update();
    app.update();
    assert_same(&app, &target, "material#lighter");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_color_transparent() {
    let (mut app, target) = configure_app();
    root(&mut app).material.color = Color::WHITE.with_alpha(0.5);
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "material#alpha", 10, 1);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_shader() {
    let (mut app, target) = configure_app();
    let shader = root(&mut app).red_shader.to_ref();
    root(&mut app).material.shader = shader;
    app.update();
    app.update();
    assert_same(&app, &target, "material#red");
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    app.update();
    let target = root(&mut app).target.to_ref();
    assert_same(&app, &target, "material#default");
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    texture: Glob<Res<Texture>>,
    shader: ShaderGlob<TestMaterial>,
    red_shader: ShaderGlob<TestMaterial>,
    material: Mat<TestMaterial>,
    model: Model2D<TestMaterial>,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let texture = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let red_shader = ShaderGlob::from_app(app);
        let material = TestMaterial::new(&texture, &shader).into_mat(app);
        let model = Model2D::new(app, material.glob());
        Self {
            texture,
            shader,
            red_shader,
            material,
            model,
            target,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.texture
            .updater()
            .path("../tests/assets/opaque-texture.png")
            .for_inner(app, |inner, app| {
                inner.updater().is_smooth(false).apply(app)
            })
            .apply(app);
        self.shader
            .updater()
            .path("../tests/assets/simple.wgsl")
            .apply(app);
        self.red_shader
            .updater()
            .path("../tests/assets/red.wgsl")
            .apply(app);
        self.model.size = Vec2::ONE * 0.5;
        self.model.camera = self.target.get(app).camera.glob().to_ref();
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
        self.model.update(app);
    }
}

struct TestMaterial {
    color: Color,
    textures: Vec<GlobRef<Res<Texture>>>,
    shader: ShaderGlobRef<Self>,
}

impl Material for TestMaterial {
    type Data = TestMaterialData;
    type InstanceData = ();

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<Res<Texture>>> {
        self.textures.clone()
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }

    fn data(&self) -> Self::Data {
        TestMaterialData {
            color: self.color.into(),
        }
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}

impl TestMaterial {
    fn new(texture: &Glob<Res<Texture>>, shader: &ShaderGlob<Self>) -> Self {
        Self {
            color: Color::DARK_GRAY,
            textures: vec![texture.to_ref()],
            shader: shader.to_ref(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData {
    color: [f32; 4],
}
