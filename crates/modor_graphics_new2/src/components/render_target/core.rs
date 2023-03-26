use crate::data::size::NonZeroSize;
use crate::{Color, RendererInner, Texture, TextureTargetBuffer};
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
    pub(crate) fn new(size: NonZeroSize, renderer: &RendererInner) -> Self {
        Self {
            size,
            depth_buffer_view: Self::create_depth_buffer_view(size, renderer),
            encoder: None,
            texture: None,
        }
    }

    pub(crate) fn size(&self) -> NonZeroSize {
        self.size
    }

    pub(crate) fn update(&mut self, size: NonZeroSize, renderer: &RendererInner) {
        if self.size != size {
            self.size = size;
            self.depth_buffer_view = Self::create_depth_buffer_view(self.size, renderer);
        }
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        background_color: Color,
        renderer: &RendererInner,
        view: TextureView,
    ) -> RenderPass<'_> {
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_render_encoder"),
        };
        self.encoder
            .insert(renderer.device.create_command_encoder(&descriptor))
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

    pub(crate) fn submit_command_queue(
        &mut self,
        texture_buffer: Option<&TextureTargetBuffer>,
        texture: Option<&Texture>,
        renderer: &RendererInner,
    ) {
        let mut encoder = self
            .encoder
            .take()
            .expect("internal error: encoder not initialized");
        if let (Some(texture_buffer), Some(texture)) = (texture_buffer, texture) {
            texture_buffer.copy_texture_to_buffer(texture.inner(), &mut encoder);
        }
        renderer.queue.submit(Some(encoder.finish()));
    }

    fn create_depth_buffer_view(size: NonZeroSize, renderer: &RendererInner) -> TextureView {
        let texture = renderer.device.create_texture(&TextureDescriptor {
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
        });
        texture.create_view(&TextureViewDescriptor::default())
    }
}
