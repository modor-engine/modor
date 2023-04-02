use crate::components::render_target::core::TargetCore;
use crate::{Color, GpuContext, Texture};
use wgpu::{RenderPass, TextureViewDescriptor};

#[derive(Debug)]
pub(crate) struct TextureTarget {
    core: TargetCore,
}

impl TextureTarget {
    pub(crate) fn new(texture: &Texture, context: &GpuContext) -> Self {
        let size = texture.inner().size;
        Self {
            core: TargetCore::new(size, context),
        }
    }

    pub(crate) fn core(&self) -> &TargetCore {
        &self.core
    }

    pub(crate) fn updated(mut self, texture: &Texture, context: &GpuContext) -> Self {
        self.core.update(texture.inner().size, context);
        self
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        texture: &Texture,
        background_color: Color,
        context: &GpuContext,
    ) -> RenderPass<'_> {
        let view = texture
            .inner()
            .texture
            .create_view(&TextureViewDescriptor::default());
        self.core.begin_render_pass(background_color, context, view)
    }

    pub(crate) fn end_render_pass(&mut self, context: &GpuContext) {
        self.core.submit_command_queue(context);
    }
}
