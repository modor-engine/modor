use modor::log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::{Res, ResUpdater};
use modor_graphics::{
    bytemuck, IntoMat, Mat, Material, MaterialGlobRef, Model2D, Model2DGlob, ShaderGlob,
    ShaderGlobRef, ShaderUpdater, Texture, TextureUpdater,
};
use std::collections::HashMap;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    texture: Glob<Res<Texture>>,
    shader: ShaderGlob<BlurMaterial>,
    material: Mat<BlurMaterial>,
    sprites: Vec<Sprite>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let texture = Glob::from_app(app);
        let shader = ShaderGlob::from_app(app);
        let material = BlurMaterial::new(&texture, &shader).into_mat(app);
        Self {
            sprites: vec![
                Sprite::new(app, Vec2::new(-0.25, 0.25), 0, material.glob()),
                Sprite::new(app, Vec2::new(0.25, 0.25), 3, material.glob()),
                Sprite::new(app, Vec2::new(-0.25, -0.25), 6, material.glob()),
                Sprite::new(app, Vec2::new(0.25, -0.25), 9, material.glob()),
            ],
            texture,
            shader,
            material,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        TextureUpdater::default()
            .res(ResUpdater::default().path("smiley.png"))
            .apply(app, &self.texture);
        ShaderUpdater::default()
            .res(ResUpdater::default().path("blur.wgsl"))
            .apply(app, &self.shader);
    }

    fn update(&mut self, app: &mut App) {
        self.material.update(app);
        for sprite in &mut self.sprites {
            sprite.update(app);
        }
    }
}

struct Sprite {
    model: Model2D<BlurMaterial>,
}

impl Sprite {
    fn new(
        app: &mut App,
        position: Vec2,
        sample_count: u32,
        material: MaterialGlobRef<BlurMaterial>,
    ) -> Self {
        let model = Model2D::new(app, material)
            .with_position(position)
            .with_size(Vec2::ONE * 0.4);
        app.get_mut::<SpriteProperties>()
            .sample_counts
            .insert(model.glob().index(), sample_count);
        Self { model }
    }

    fn update(&mut self, app: &mut App) {
        self.model.update(app);
    }
}

#[derive(Default, State)]
struct SpriteProperties {
    sample_counts: HashMap<usize, u32>,
}

struct BlurMaterial {
    blur_factor: f32,
    texture: GlobRef<Res<Texture>>,
    shader: ShaderGlobRef<Self>,
}

impl Material for BlurMaterial {
    type Data = BlurMaterialData;
    type InstanceData = BlurInstanceData;

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<Res<Texture>>> {
        vec![self.texture.clone()]
    }

    fn is_transparent(&self) -> bool {
        false
    }

    fn data(&self) -> Self::Data {
        BlurMaterialData {
            blur_factor: self.blur_factor,
            padding1: [0.],
            padding2: [0., 0.],
        }
    }

    fn instance_data(app: &mut App, model: &Glob<Model2DGlob>) -> Self::InstanceData {
        let sample_counts = &app.get_mut::<SpriteProperties>().sample_counts;
        BlurInstanceData {
            sample_count: sample_counts.get(&model.index()).copied().unwrap_or(0),
        }
    }
}

impl BlurMaterial {
    fn new(texture: &Glob<Res<Texture>>, shader: &ShaderGlob<Self>) -> Self {
        Self {
            blur_factor: 0.005,
            texture: texture.to_ref(),
            shader: shader.to_ref(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurMaterialData {
    blur_factor: f32,
    padding1: [f32; 1],
    padding2: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurInstanceData {
    sample_count: u32,
}
