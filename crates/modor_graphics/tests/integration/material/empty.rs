#![allow(clippy::trailing_empty_array)]

use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    MatGlob, MatUpdater, Material, Model2D, Model2DGlob, ShaderGlob, ShaderUpdater, Size, Texture,
    TextureSource, TextureUpdater,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};

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
    material: MatGlob<TestMaterial>,
    model: Model2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let material = MatGlob::from_app(app);
        let model = Model2D::new(app).with_material(material.to_ref());
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
        ShaderUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/red.wgsl"))
            .apply(app, &self.shader);
        MatUpdater::default()
            .shader(self.shader.to_ref())
            .apply(app, &self.material);
        self.model.size = Vec2::ONE * 0.5;
        self.model.camera = self.target.get(app).camera().glob().to_ref();
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
    }

    fn update(&mut self, app: &mut App) {
        self.model.update(app);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod, FromApp)]
struct TestMaterial;

impl Material for TestMaterial {
    type InstanceData = ();

    fn init(_app: &mut App, _glob: &MatGlob<Self>) {}

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}
