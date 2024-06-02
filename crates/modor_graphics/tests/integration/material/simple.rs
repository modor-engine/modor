use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::{assert_max_component_diff, assert_same};
use modor_graphics::{
    Color, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, Size, Texture, TextureGlob,
    TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_textures_less_than_shader() {
    let (mut app, target) = configure_app();
    root(&mut app).material.textures = vec![];
    app.update();
    assert_same(&mut app, &target, "material#no_texture");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_textures_more_than_shader() {
    let (mut app, target) = configure_app();
    let texture = root(&mut app).texture.glob().clone();
    root(&mut app).material.textures = vec![texture.clone(), texture];
    app.update();
    assert_same(&mut app, &target, "material#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_color_opaque() {
    let (mut app, target) = configure_app();
    root(&mut app).material.color = Color::WHITE;
    app.update();
    assert_same(&mut app, &target, "material#lighter");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_color_transparent() {
    let (mut app, target) = configure_app();
    root(&mut app).material.color = Color::WHITE.with_alpha(0.5);
    app.update();
    assert_max_component_diff(&mut app, &target, "material#alpha", 10, 1);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_shader() {
    let (mut app, target) = configure_app();
    let shader = root(&mut app).red_shader.glob();
    root(&mut app).material.shader = shader;
    app.update();
    assert_same(&mut app, &target, "material#red");
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    Root::wait_resources(&mut app);
    let target = root(&mut app).target.glob().clone();
    assert_same(&mut app, &target, "material#default");
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    texture: Res<Texture>,
    shader: Res<Shader<TestMaterial>>,
    red_shader: Res<Shader<TestMaterial>>,
    material: Mat<TestMaterial>,
    model: Model2D<TestMaterial>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut target =
            Res::<Texture>::from_source(ctx, "main", TextureSource::Size(Size::new(30, 20)));
        target.is_target_enabled = true;
        target.is_buffer_enabled = true;
        let mut texture =
            Res::<Texture>::from_path(ctx, "main", "../tests/assets/opaque-texture.png");
        texture.is_smooth = false;
        let shader = Res::<Shader<_>>::from_path(ctx, "main", "../tests/assets/simple.wgsl");
        let red_shader = Res::<Shader<_>>::from_path(ctx, "main", "../tests/assets/red.wgsl");
        let material_data = TestMaterial {
            color: Color::DARK_GRAY,
            textures: vec![texture.glob().clone()],
            shader: shader.glob(),
        };
        let material = Mat::new(ctx, "main", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.size = Vec2::ONE * 0.5;
        model.camera = target.camera.glob().clone();
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

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target);
        wait_resource(app, |r: &Self| &r.texture);
        wait_resource(app, |r: &Self| &r.shader);
        wait_resource(app, |r: &Self| &r.red_shader);
    }
}

#[derive(Node, Visit)]
struct TestMaterial {
    color: Color,
    textures: Vec<GlobRef<TextureGlob>>,
    shader: ShaderGlobRef<Self>,
}

impl Material for TestMaterial {
    type Data = TestMaterialData;
    type InstanceData = ();

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
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

    fn instance_data(_ctx: &mut Context<'_>, _model: &GlobRef<Model2DGlob>) -> Self::InstanceData {}
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData {
    color: [f32; 4],
}
