#![allow(clippy::trailing_empty_array)]

use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    IntoMat, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, Size, Texture,
    TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};

#[modor::test(disabled(windows, macos, android, wasm))]
fn deref() {
    let mut app = App::new::<Root>(Level::Info);
    let shader = Shader::new(&mut app.ctx(), "main")
        .load_from_path(&mut app.ctx(), "../tests/assets/red.wgsl");
    let material = TestMaterial::new(&shader).into_mat(&mut app.ctx(), "main");
    assert_eq!(material.shader, shader.glob());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_material_empty_struct() {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target.glob().clone();
    assert_same(&mut app, &target, "material#red");
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    shader: Res<Shader<TestMaterial>>,
    material: Mat<TestMaterial>,
    model: Model2D<TestMaterial>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let target = Texture::new(ctx, "target")
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(ctx, TextureSource::Size(Size::new(30, 20)));
        let shader = Shader::new(ctx, "main").load_from_path(ctx, "../tests/assets/red.wgsl");
        let material = TestMaterial::new(&shader).into_mat(ctx, "main");
        let model = Model2D::new(ctx, material.glob())
            .with_size(Vec2::ONE * 0.5)
            .with_camera(target.camera.glob().clone());
        Self {
            shader,
            material,
            model,
            target,
        }
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

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
        vec![]
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn data(&self) -> Self::Data {
        TestMaterialData
    }

    fn instance_data(_ctx: &mut Context<'_>, _model: &GlobRef<Model2DGlob>) -> Self::InstanceData {}
}

impl TestMaterial {
    fn new(shader: &Res<Shader<Self>>) -> Self {
        Self {
            shader: shader.glob(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData;
