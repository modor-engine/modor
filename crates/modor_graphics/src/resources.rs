use crate::mesh::Mesh;
use crate::{
    Camera2D, DefaultMaterial2D, Shader, ShaderSource, Size, Texture, TextureSource, Window,
};
use modor::{Context, Node, RootNode, Visit};
use modor_resources::Res;

#[non_exhaustive]
#[derive(Debug, Node, Visit)]
pub struct GraphicsResources {
    pub window_camera: Camera2D,
    pub(crate) rectangle_mesh: Mesh,
    pub(crate) default_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) ellipse_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) white_texture: Res<Texture>,
}

impl RootNode for GraphicsResources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let window_target = ctx.get_mut::<Window>().target.glob().clone();
        Self {
            window_camera: Camera2D::new(ctx, "window(modor_graphics)", vec![window_target]),
            rectangle_mesh: Mesh::rectangle(ctx),
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
            white_texture: Res::from_source(
                ctx,
                "white(modor_graphics)",
                TextureSource::Size(Size::ONE),
            ),
        }
    }
}
