use modor::log::Level;
use modor::{Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{
    bytemuck, IntoMat, Mat, Material, MaterialGlobRef, Model2D, Model2DGlob, Shader, ShaderGlobRef,
    Texture, TextureGlob,
};
use std::collections::HashMap;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    texture: Res<Texture>,
    shader: Res<Shader<BlurMaterial>>,
    material: Mat<BlurMaterial>,
    sprites: Vec<Sprite>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let texture = Texture::new(ctx, "smiley").load_from_path(ctx, "smiley.png");
        let shader = Shader::new(ctx, "blur").load_from_path(ctx, "blur.wgsl");
        let material = BlurMaterial::new(&texture, &shader).into_mat(ctx, "blur-default");
        Self {
            sprites: vec![
                Sprite::new(ctx, Vec2::new(-0.25, 0.25), 0, material.glob()),
                Sprite::new(ctx, Vec2::new(0.25, 0.25), 3, material.glob()),
                Sprite::new(ctx, Vec2::new(-0.25, -0.25), 6, material.glob()),
                Sprite::new(ctx, Vec2::new(0.25, -0.25), 9, material.glob()),
            ],
            texture,
            shader,
            material,
        }
    }
}

#[derive(Node, Visit)]
struct Sprite {
    model: Model2D<BlurMaterial>,
}

impl Sprite {
    fn new(
        ctx: &mut Context<'_>,
        position: Vec2,
        sample_count: u32,
        material: MaterialGlobRef<BlurMaterial>,
    ) -> Self {
        let model = Model2D::new(ctx, material)
            .with_position(position)
            .with_size(Vec2::ONE * 0.4);
        ctx.get_mut::<SpriteProperties>()
            .sample_counts
            .insert(model.glob().index(), sample_count);
        Self { model }
    }
}

#[derive(Default, RootNode, Node, Visit)]
struct SpriteProperties {
    sample_counts: HashMap<usize, u32>,
}

struct BlurMaterial {
    blur_factor: f32,
    texture: GlobRef<TextureGlob>,
    shader: ShaderGlobRef<Self>,
}

impl Material for BlurMaterial {
    type Data = BlurMaterialData;
    type InstanceData = BlurInstanceData;

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
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

    fn instance_data(ctx: &mut Context<'_>, model: &GlobRef<Model2DGlob>) -> Self::InstanceData {
        let sample_counts = &ctx.get_mut::<SpriteProperties>().sample_counts;
        BlurInstanceData {
            sample_count: sample_counts.get(&model.index()).copied().unwrap_or(0),
        }
    }
}

impl BlurMaterial {
    fn new(texture: &Res<Texture>, shader: &Res<Shader<Self>>) -> Self {
        Self {
            blur_factor: 0.005,
            texture: texture.glob().clone(),
            shader: shader.glob(),
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
