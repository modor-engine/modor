use crate::mesh::Mesh;
use crate::{DefaultMaterial2D, ShaderGlob, ShaderSource, Size, Texture, TextureSource};
use modor::{App, FromApp, Glob, State};
use modor_resources::Res;

#[non_exhaustive]
#[derive(Debug, FromApp)]
pub(crate) struct Resources {
    pub(crate) rectangle_mesh: Glob<Mesh>,
    pub(crate) empty_shader: ShaderGlob<DefaultMaterial2D>,
    pub(crate) default_shader: ShaderGlob<DefaultMaterial2D>,
    pub(crate) ellipse_shader: ShaderGlob<DefaultMaterial2D>,
    pub(crate) white_texture: Glob<Res<Texture>>,
}

impl State for Resources {
    fn init(&mut self, app: &mut App) {
        self.empty_shader
            .updater()
            .source(ShaderSource::String(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/empty.wgsl")).into(),
            ))
            .apply(app);
        self.default_shader
            .updater()
            .source(ShaderSource::String(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")).into(),
            ))
            .apply(app);
        self.ellipse_shader
            .updater()
            .source(ShaderSource::String(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")).into(),
            ))
            .apply(app);
        self.white_texture
            .updater()
            .source(TextureSource::Size(Size::ONE))
            .apply(app);
    }
}
