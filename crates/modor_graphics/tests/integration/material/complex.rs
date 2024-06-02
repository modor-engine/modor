use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, Size, Texture, TextureGlob,
    TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_instance_data() {
    let mut app = App::new::<Root>(Level::Info);
    Root::wait_resources(&mut app);
    let target = root(&mut app).target.glob().clone();
    assert_same(&mut app, &target, "material#instances");
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    texture: Res<Texture>,
    shader: Res<Shader<TestMaterial>>,
    material: Mat<TestMaterial>,
    model1: Model2D<TestMaterial>,
    model2: Model2D<TestMaterial>,
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
        let shader = Res::<Shader<_>>::from_path(ctx, "main", "../tests/assets/complex.wgsl");
        let material_data = TestMaterial {
            color: Color::DARK_GRAY,
            texture: texture.glob().clone(),
            shader: shader.glob(),
        };
        let material = Mat::new(ctx, "main", material_data);
        let mut model1 = Model2D::new(ctx, material.glob());
        model1.position = Vec2::new(-0.25, 0.);
        model1.size = Vec2::new(0.25, 0.5);
        model1.camera = target.camera.glob().clone();
        let mut model2 = Model2D::new(ctx, material.glob());
        model2.position = Vec2::new(0.25, 0.);
        model2.size = Vec2::new(0.25, 0.5);
        model2.camera = target.camera.glob().clone();
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

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target);
        wait_resource(app, |r: &Self| &r.texture);
        wait_resource(app, |r: &Self| &r.shader);
    }
}

#[derive(Node, Visit)]
struct TestMaterial {
    color: Color,
    texture: GlobRef<TextureGlob>,
    shader: ShaderGlobRef<Self>,
}

impl Material for TestMaterial {
    type Data = TestMaterialData;
    type InstanceData = TestInstanceData;

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
        vec![self.texture.clone()]
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }

    fn data(&self) -> Self::Data {
        TestMaterialData {
            color: self.color.into(),
        }
    }

    fn instance_data(_ctx: &mut Context<'_>, model: &GlobRef<Model2DGlob>) -> Self::InstanceData {
        vec![
            TestInstanceData {
                color: [0., 0., 1., 1.],
            },
            TestInstanceData {
                color: [0., 1., 0., 1.],
            },
        ][model.index()]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestMaterialData {
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TestInstanceData {
    color: [f32; 4],
}
