use crate::{Font, FontSource, TextMaterial2D};
use modor::{App, Node, RootNode, Visit};
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Shader, ShaderSource};

#[derive(Node, Visit)]
pub(crate) struct TextResources {
    pub(crate) text_shader: Res<Shader<TextMaterial2D>>,
    pub(crate) default_font: Res<Font>,
}

impl RootNode for TextResources {
    fn on_create(app: &mut App) -> Self {
        Self {
            text_shader: Shader::new(app, "text-2d(modor_text)").load_from_source(
                app,
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/text.wgsl")).into(),
                ),
            ),
            default_font: Font::new(app, "default(modor_text)").load_from_source(
                app,
                FontSource::Bytes(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/res/Roboto-Regular.ttf"
                ))),
            ),
        }
    }
}