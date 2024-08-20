use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, MatGlob, MatUpdater, Material, Model2D, Model2DGlob, ShaderGlob, ShaderUpdater, Size,
    Texture, TextureSource, TextureUpdater,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_instance_data() {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    app.update();
    let target = root(&mut app).target.to_ref();
    assert_same(&app, &target, "material#instances");
    app.update();
    assert_same(&app, &target, "material#instances");
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    texture: Glob<Res<Texture>>,
    shader: ShaderGlob<TestMaterial>,
    material: MatGlob<TestMaterial>,
    model1: Model2D,
    model2: Model2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let texture = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let material = MatGlob::from_app(app);
        let model1 = Model2D::new(app).with_material(material.to_ref());
        let model2 = Model2D::new(app).with_material(material.to_ref());
        Self {
            texture,
            shader,
            material,
            model1,
            model2,
            target,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        TextureUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/opaque-texture.png"))
            .is_smooth(false)
            .apply(app, &self.texture);
        ShaderUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/complex.wgsl"))
            .apply(app, &self.shader);
        MatUpdater::default()
            .textures(vec![self.texture.to_ref()])
            .shader(self.shader.to_ref())
            .apply(app, &self.material);
        self.model1.position = Vec2::new(-0.25, 0.);
        self.model1.size = Vec2::new(0.25, 0.5);
        self.model1.camera = self.target.get(app).camera().glob().to_ref();
        self.model2.position = Vec2::new(0.25, 0.);
        self.model2.size = Vec2::new(0.25, 0.5);
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
            color: Color::DARK_GRAY.into(),
        }
    }
}

impl Material for TestMaterial {
    type InstanceData = TestMaterialInstance;

    fn init(_app: &mut App, _glob: &MatGlob<Self>) {}

    fn instance_data(_app: &mut App, model: &Glob<Model2DGlob>) -> Self::InstanceData {
        vec![
            TestMaterialInstance {
                color: [0., 0., 1., 1.],
            },
            TestMaterialInstance {
                color: [0., 1., 0., 1.],
            },
        ][model.index()]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialInstance {
    color: [f32; 4],
}
