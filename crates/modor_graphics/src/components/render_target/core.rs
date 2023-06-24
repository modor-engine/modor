use crate::data::size::NonZeroSize;
use crate::{Color, GpuContext};
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Extent3d, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

#[derive(Debug)]
pub(crate) struct TargetCore {
    size: NonZeroSize,
    depth_buffer_view: TextureView,
    encoder: Option<CommandEncoder>,
    texture: Option<TextureView>,
}

impl TargetCore {
    pub(crate) fn new(size: NonZeroSize, context: &GpuContext) -> Self {
        Self {
            size,
            depth_buffer_view: Self::create_depth_buffer_view(size, context),
            encoder: None,
            texture: None,
        }
    }

    pub(crate) fn size(&self) -> NonZeroSize {
        self.size
    }

    pub(crate) fn update(&mut self, size: NonZeroSize, context: &GpuContext) {
        if self.size != size {
            self.size = size;
            self.depth_buffer_view = Self::create_depth_buffer_view(self.size, context);
        }
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        background_color: Color,
        context: &GpuContext,
        view: TextureView,
    ) -> RenderPass<'_> {
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_render_encoder"),
        };
        self.encoder
            .insert(context.device.create_command_encoder(&descriptor))
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("modor_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: self.texture.insert(view),
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(background_color.into()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            })
    }

    pub(crate) fn submit_command_queue(&mut self, context: &GpuContext) {
        let encoder = self
            .encoder
            .take()
            .expect("internal error: encoder not initialized");
        context.queue.submit(Some(encoder.finish()));
    }

    fn create_depth_buffer_view(size: NonZeroSize, context: &GpuContext) -> TextureView {
        let texture = context.device.create_texture(&TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Depth32Float],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }
}