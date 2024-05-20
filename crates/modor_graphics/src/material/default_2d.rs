use crate::resources::GraphicsResources;
use crate::texture::glob::TextureGlob;
use crate::{Color, Material, Model2DGlob, Shader, ShaderGlobRef, ShaderSource};
use internal::DefaultMaterial2DData;
use modor::{Context, GlobRef, Node, RootNode, Visit};
use modor_input::modor_math::Vec2;
use modor_resources::Res;

#[derive(Debug, Node, Visit)]
pub struct DefaultMaterial2D {
    pub color: Color,
    pub texture: GlobRef<TextureGlob>,
    pub texture_position: Vec2,
    pub texture_size: Vec2,
    pub is_ellipse: bool,
    default_shader: ShaderGlobRef<Self>,
    ellipse_shader: ShaderGlobRef<Self>,
}

impl Material for DefaultMaterial2D {
    type Data = DefaultMaterial2DData;
    type InstanceData = ();

    fn shader(&self) -> ShaderGlobRef<Self> {
        if self.is_ellipse {
            self.ellipse_shader.clone()
        } else {
            self.default_shader.clone()
        }
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
        vec![self.texture.clone()]
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }

    fn data(&self) -> Self::Data {
        DefaultMaterial2DData {
            color: self.color.into(),
            texture_part_position: [self.texture_position.x, self.texture_position.y],
            texture_part_size: [self.texture_size.x, self.texture_size.y],
        }
    }

    fn instance_data(_ctx: &mut Context<'_>, _model: &GlobRef<Model2DGlob>) -> Self::InstanceData {}
}

impl DefaultMaterial2D {
    pub fn new(ctx: &mut Context<'_>) -> Self {
        let resources = ctx.get_mut::<GraphicsResources>();
        Self {
            color: Color::WHITE,
            texture: resources.white_texture.glob().clone(),
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            is_ellipse: false,
            default_shader: resources.default_shader.glob(),
            ellipse_shader: resources.ellipse_shader.glob(),
        }
    }
}

#[derive(Debug, Node, Visit)]
struct DefaultMaterial2DResources {
    default_shader: Res<Shader<DefaultMaterial2D>>,
    ellipse_shader: Res<Shader<DefaultMaterial2D>>,
}

impl RootNode for DefaultMaterial2DResources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            default_shader: Res::from_source(
                ctx,
                "default(modor_graphics)",
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")).into(),
                ),
            ),
            ellipse_shader: Res::from_source(
                ctx,
                "ellipse(modor_graphics)",
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")).into(),
                ),
            ),
        }
    }
}

pub mod internal {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct DefaultMaterial2DData {
        pub(crate) color: [f32; 4],
        pub(crate) texture_part_position: [f32; 2],
        pub(crate) texture_part_size: [f32; 2],
    }
}
