use bytemuck::{Pod, Zeroable};
use log::Level;
use modor::{App, Glob, GlobRef, Node, RootNode};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, IntoMat, Mat, Material, Model2D, Model2DGlob, Shader, ShaderGlobRef, Size, Texture,
    TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_instance_data() {
    let mut app = App::new::<Root>(Level::Info);
    wait_resources(&mut app);
    let target = root(&mut app).target.glob().to_ref();
    assert_same(&app, &target, "material#instances");
    app.update();
    assert_same(&app, &target, "material#instances");
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    texture: Res<Texture>,
    shader: Res<Shader<TestMaterial>>,
    material: Mat<TestMaterial>,
    model1: Model2D<TestMaterial>,
    model2: Model2D<TestMaterial>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(30, 20)));
        let texture = Texture::new(app)
            .with_is_smooth(false)
            .load_from_path(app, "../tests/assets/opaque-texture.png");
        let shader = Shader::new(app).load_from_path(app, "../tests/assets/complex.wgsl");
        let material = TestMaterial::new(&texture, &shader).into_mat(app);
        let model1 = Model2D::new(app, material.glob())
            .with_position(Vec2::new(-0.25, 0.))
            .with_size(Vec2::new(0.25, 0.5))
            .with_camera(target.camera.glob().to_ref());
        let model2 = Model2D::new(app, material.glob())
            .with_position(Vec2::new(0.25, 0.))
            .with_size(Vec2::new(0.25, 0.5))
            .with_camera(target.camera.glob().to_ref());
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

impl Node for Root {
    fn update(&mut self, app: &mut App) {
        self.texture.update(app);
        self.shader.update(app);
        self.material.update(app);
        self.model1.update(app);
        self.model2.update(app);
        self.target.update(app);
    }
}

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

    fn instance_data(_app: &mut App, model: &Glob<Model2DGlob>) -> Self::InstanceData {
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

impl TestMaterial {
    fn new(texture: &Res<Texture>, shader: &Res<Shader<Self>>) -> Self {
        Self {
            color: Color::DARK_GRAY,
            texture: texture.glob().to_ref(),
            shader: shader.glob(),
        }
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
