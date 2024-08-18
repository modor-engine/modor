use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{
    Color, MatGlob, MatUpdater, Material, Model2D, Model2DGlob, ShaderGlob, ShaderUpdater, Size,
    Texture, TextureSource, TextureUpdater,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_textures_less_than_shader() {
    let (mut app, target) = configure_app();
    app.take::<Root, _>(|root, app| {
        MatUpdater::default()
            .textures(vec![])
            .apply(app, &root.material);
    });
    app.update();
    app.update();
    assert_same(&app, &target, "material#no_texture");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_textures_more_than_shader() {
    let (mut app, target) = configure_app();
    let texture = root(&mut app).texture.to_ref();
    app.take::<Root, _>(|root, app| {
        MatUpdater::default()
            .textures(vec![texture.clone(), texture])
            .apply(app, &root.material);
    });
    app.update();
    assert_same(&app, &target, "material#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_color_opaque() {
    let (mut app, target) = configure_app();
    app.take::<Root, _>(|root, app| {
        MatUpdater::default()
            .data(TestMaterial {
                color: Color::WHITE.into(),
            })
            .apply(app, &root.material);
    });
    app.update();
    app.update();
    assert_same(&app, &target, "material#lighter");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_color_transparent() {
    let (mut app, target) = configure_app();
    app.take::<Root, _>(|root, app| {
        MatUpdater::default()
            .data(TestMaterial {
                color: Color::WHITE.with_alpha(0.5).into(),
            })
            .is_transparent(true)
            .apply(app, &root.material);
    });
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "material#alpha", 10, 1);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_shader() {
    let (mut app, target) = configure_app();
    let shader = root(&mut app).red_shader.to_ref();
    app.take::<Root, _>(|root, app| {
        MatUpdater::default()
            .shader(shader)
            .apply(app, &root.material);
    });
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
    material: MatGlob<TestMaterial>,
    model: Model2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let texture = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let red_shader = ShaderGlob::from_app(app);
        let material = MatGlob::from_app(app);
        let model = Model2D::new(app).with_material(material.to_ref());
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
        TextureUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/opaque-texture.png"))
            .is_smooth(false)
            .apply(app, &self.texture);
        ShaderUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/simple.wgsl"))
            .apply(app, &self.shader);
        ShaderUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/red.wgsl"))
            .apply(app, &self.red_shader);
        MatUpdater::default()
            .shader(self.shader.to_ref())
            .textures(vec![self.texture.to_ref()])
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
    type InstanceData = ();

    fn init(self, _app: &mut App, _glob: &MatGlob<Self>) {}

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}
