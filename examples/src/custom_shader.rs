use modor::log::Level;
use modor::{Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_resources::Res;
use modor_graphics::{bytemuck, Mat, Material, Model2D, Model2DGlob, Shader, Texture, TextureGlob};
use std::collections::HashMap;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    sprites: Vec<Sprite>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            sprites: vec![
                Sprite::new(ctx, Vec2::new(-0.25, 0.25), 0),
                Sprite::new(ctx, Vec2::new(0.25, 0.25), 3),
                Sprite::new(ctx, Vec2::new(-0.25, -0.25), 6),
                Sprite::new(ctx, Vec2::new(0.25, -0.25), 9),
            ],
        }
    }
}

#[derive(Node, Visit)]
struct Sprite {
    model: Model2D<BlurMaterial>,
}

impl Sprite {
    fn new(ctx: &mut Context<'_>, position: Vec2, sample_count: u32) -> Self {
        let material = ctx.get_mut::<Materials>().blur_material.glob();
        let mut model = Model2D::new(ctx, material);
        model.position = position;
        model.size = Vec2::ONE * 0.4;
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

#[derive(Node, Visit)]
struct Resources {
    blur_shader: Res<Shader<BlurMaterial>>,
    texture: Res<Texture>,
}

impl RootNode for Resources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            blur_shader: Res::from_path(ctx, "blur", "blur.wgsl"),
            texture: Res::<Texture>::from_path(ctx, "smiley", "smiley.png"),
        }
    }
}

#[derive(Node, Visit)]
struct Materials {
    blur_material: Mat<BlurMaterial>,
}

impl RootNode for Materials {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            blur_material: Mat::new(ctx, "blur-default", BlurMaterial { blur_factor: 0.005 }),
        }
    }
}

struct BlurMaterial {
    blur_factor: f32,
}

impl Material for BlurMaterial {
    type Data = BlurMaterialData;
    type InstanceData = BlurInstanceData;

    fn shader<'a>(&self, ctx: &'a mut Context<'_>) -> &'a Res<Shader<Self>> {
        &ctx.get_mut::<Resources>().blur_shader
    }

    fn textures(&self, ctx: &mut Context<'_>) -> Vec<GlobRef<TextureGlob>> {
        vec![ctx.get_mut::<Resources>().texture.glob().clone()]
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
