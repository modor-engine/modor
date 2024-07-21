use crate::mesh::Mesh;
use crate::{DefaultMaterial2D, Shader, ShaderSource, Size, Texture, TextureSource};
use modor::{App, Node, RootNode, Visit};
use modor_resources::{Res, ResLoad};

#[non_exhaustive]
#[derive(Debug, Node, Visit)]
pub(crate) struct Resources {
    pub(crate) rectangle_mesh: Mesh,
    pub(crate) empty_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) default_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) ellipse_shader: Res<Shader<DefaultMaterial2D>>,
    pub(crate) white_texture: Res<Texture>,
}

impl RootNode for Resources {
    fn on_create(app: &mut App) -> Self {
        Self {
            rectangle_mesh: Mesh::rectangle(app),
            empty_shader: Shader::new(app).load_from_source(
                app,
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/empty.wgsl")).into(),
                ),
            ),
            default_shader: Shader::new(app).load_from_source(
                app,
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")).into(),
                ),
            ),
            ellipse_shader: Shader::new(app).load_from_source(
                app,
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")).into(),
                ),
            ),
            white_texture: Texture::new(app).load_from_source(app, TextureSource::Size(Size::ONE)),
        }
    }
}
