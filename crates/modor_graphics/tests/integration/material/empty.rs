#![allow(clippy::trailing_empty_array)]

use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, Size, Texture, TextureGlob,
    TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn deref() {
    let mut app = App::new::<Root>(Level::Info);
    let shader = Res::<Shader<_>>::from_path(&mut app.ctx(), "main", "../tests/assets/red.wgsl");
    let material_data = TestMaterial {
        shader: shader.glob(),
    };
    let material = Mat::new(&mut app.ctx(), "main", material_data);
    assert_eq!(material.shader, shader.glob());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_material_empty_struct() {
    let mut app = App::new::<Root>(Level::Info);
    Root::wait_resources(&mut app);
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
        let mut target =
            Res::<Texture>::from_source(ctx, "main", TextureSource::Size(Size::new(30, 20)));
        target.is_target_enabled = true;
        target.is_buffer_enabled = true;
        let shader = Res::<Shader<_>>::from_path(ctx, "main", "../tests/assets/red.wgsl");
        let material_data = TestMaterial {
            shader: shader.glob(),
        };
        let material = Mat::new(ctx, "main", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.size = Vec2::ONE * 0.5;
        model.camera = target.camera.glob().clone();
        Self {
            shader,
            material,
            model,
            target,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target);
        wait_resource(app, |r: &Self| &r.shader);
    }
}

#[derive(Node, Visit)]
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

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData;
