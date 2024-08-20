use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::{Res, ResUpdater};
use modor_graphics::{
    bytemuck, MatGlob, MatUpdater, Material, Model2D, Model2DGlob, ShaderGlob, ShaderUpdater,
    Texture, TextureUpdater,
};
use std::collections::HashMap;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    sprites: Vec<Sprite>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let material = MatGlob::from_app(app);
        Self {
            sprites: vec![
                Sprite::new(app, Vec2::new(-0.25, 0.25), 0, &material),
                Sprite::new(app, Vec2::new(0.25, 0.25), 3, &material),
                Sprite::new(app, Vec2::new(-0.25, -0.25), 6, &material),
                Sprite::new(app, Vec2::new(0.25, -0.25), 9, &material),
            ],
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        for sprite in &mut self.sprites {
            sprite.update(app);
        }
    }
}

#[derive(FromApp)]
struct Resources {
    shader: ShaderGlob<BlurMaterial>,
    texture: Glob<Res<Texture>>,
}

impl State for Resources {
    fn init(&mut self, app: &mut App) {
        ShaderUpdater::default()
            .res(ResUpdater::default().path("blur.wgsl"))
            .apply(app, &self.shader);
        TextureUpdater::default()
            .res(ResUpdater::default().path("smiley.png"))
            .apply(app, &self.texture);
    }
}

struct Sprite {
    model: Model2D,
}

impl Sprite {
    fn new(
        app: &mut App,
        position: Vec2,
        sample_count: u32,
        material: &MatGlob<BlurMaterial>,
    ) -> Self {
        let model = Model2D::new(app)
            .with_material(material.to_ref())
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurMaterial {
    blur_factor: f32,
    padding1: [f32; 1],
    padding2: [f32; 2],
}

impl Default for BlurMaterial {
    fn default() -> Self {
        Self {
            blur_factor: 0.005,
            padding1: [0.],
            padding2: [0., 0.],
        }
    }
}

impl Material for BlurMaterial {
    type InstanceData = BlurInstanceData;

    fn init(app: &mut App, glob: &MatGlob<Self>) {
        let resources = app.get_mut::<Resources>();
        MatUpdater::default()
            .shader(resources.shader.to_ref())
            .textures(vec![resources.texture.to_ref()])
            .apply(app, glob);
    }

    fn instance_data(app: &mut App, model: &Glob<Model2DGlob>) -> Self::InstanceData {
        let sample_counts = &app.get_mut::<SpriteProperties>().sample_counts;
        BlurInstanceData {
            sample_count: sample_counts.get(&model.index()).copied().unwrap_or(0),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct BlurInstanceData {
    sample_count: u32,
}
