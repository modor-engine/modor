use crate::{Font, FontSource, TextMaterial2D};
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_resources::Res;
use modor_graphics::{ShaderGlob, ShaderSource};

#[derive(FromApp)]
pub(crate) struct TextResources {
    pub(crate) text_shader: ShaderGlob<TextMaterial2D>,
    pub(crate) default_font: Glob<Res<Font>>,
}

impl State for TextResources {
    fn init(&mut self, app: &mut App) {
        self.text_shader
            .updater()
            .source(ShaderSource::String(
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/text.wgsl")).into(),
            ))
            .apply(app);
        self.default_font
            .updater()
            .source(FontSource::Bytes(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/res/Roboto-Regular.ttf"
            ))))
            .apply(app);
    }
}
