use crate::mesh::Mesh;
use crate::{DefaultMaterial2D, Shader, ShaderSource, Size, Texture, TextureSource};
use modor::{Context, Node, RootNode, Visit};
use modor_resources::{Res, ResLoad};

#[non_exhaustive]
#[derive(Debug, Node, Visit)]
pub(crate) struct GraphicsResources {
    pub(crate) rectangle_mesh: Mesh,
    pub(crate) default_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) ellipse_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) white_texture: Res<Texture>,
}

impl RootNode for GraphicsResources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            rectangle_mesh: Mesh::rectangle(ctx),
            default_shader: Shader::new(ctx, "default(modor_graphics)").load_from_source(
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")).into(),
                ),
            ),
            ellipse_shader: Shader::new(ctx, "ellipse(modor_graphics)").load_from_source(
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")).into(),
                ),
            ),
            white_texture: Texture::new(ctx, "white(modor_graphics)")
                .load_from_source(TextureSource::Size(Size::ONE)),
        }
    }
}
