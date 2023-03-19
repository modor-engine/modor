use crate::components::render_target::core::TargetCore;
use crate::{Color, Renderer, Texture, TextureTargetBuffer};
use wgpu::{RenderPass, TextureViewDescriptor};

#[derive(Debug)]
pub(crate) struct TextureTarget {
    core: TargetCore,
}

impl TextureTarget {
    pub(crate) fn new(texture: &Texture, renderer: &Renderer) -> Self {
        let size = texture.size();
        Self {
            core: TargetCore::new(size, renderer),
        }
    }

    pub(crate) fn core(&self) -> &TargetCore {
        &self.core
    }

    pub(crate) fn updated(mut self, texture: &Texture, renderer: &Renderer) -> Self {
        self.core.update(texture.size(), renderer);
        self
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        texture: &Texture,
        background_color: Color,
        renderer: &Renderer,
    ) -> RenderPass<'_> {
        let view = texture
            .inner()
            .create_view(&TextureViewDescriptor::default());
        self.core
            .begin_render_pass(background_color, renderer, view)
    }

    pub(crate) fn end_render_pass(
        &mut self,
        texture_buffer: Option<&TextureTargetBuffer>,
        texture: &Texture,
        renderer: &Renderer,
    ) {
        self.core
            .submit_command_queue(texture_buffer, Some(texture), renderer);
    }
}
