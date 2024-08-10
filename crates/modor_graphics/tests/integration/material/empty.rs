#![allow(clippy::trailing_empty_array)]

use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    IntoMat, Mat, Material, Model2D, Model2DGlob, ShaderGlob, ShaderGlobRef, Size, Texture,
    TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_material_empty_struct() {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    app.update();
    let target = root(&mut app).target.to_ref();
    assert_same(&app, &target, "material#red");
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    shader: ShaderGlob<TestMaterial>,
    material: Mat<TestMaterial>,
    model: Model2D<TestMaterial>,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let material = TestMaterial::new(&shader).into_mat(app);
        let model = Model2D::new(app, material.glob());
        Self {
            shader,
            material,
            model,
            target,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.shader
            .updater()
            .path("../tests/assets/red.wgsl")
            .apply(app);
        self.model.size = Vec2::ONE * 0.5;
        self.model.camera = self.target.get(app).camera.glob().to_ref();
        self.target
            .updater()
            .source(TextureSource::Size(Size::new(30, 20)))
            .inner(|i, _| i.is_target_enabled(true))
            .inner(|i, _| i.is_buffer_enabled(true))
            .apply(app);
    }

    fn update(&mut self, app: &mut App) {
        self.material.update(app);
        self.model.update(app);
    }
}

struct TestMaterial {
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
        false
    }

    fn data(&self) -> Self::Data {
        TestMaterialData
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}

impl TestMaterial {
    fn new(shader: &ShaderGlob<Self>) -> Self {
        Self {
            shader: shader.to_ref(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData;
