use crate::components::shader::Shader;
use crate::data::size::NonZeroSize;
use crate::{AntiAliasing, Color, GpuContext};
use std::num::NonZeroU32;
use wgpu::{
    CommandEncoder, CommandEncoderDescriptor, Extent3d, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

#[derive(Debug)]
pub(crate) struct TargetCore {
    size: NonZeroSize,
    color_buffer_view: TextureView,
    depth_buffer_view: TextureView,
    encoder: Option<CommandEncoder>,
    texture: Option<TextureView>,
    texture_format: TextureFormat,
    sample_count: u32,
}

impl TargetCore {
    pub(crate) fn new(
        size: NonZeroSize,
        anti_aliasing: Option<&AntiAliasing>,
        context: &GpuContext,
    ) -> Self {
        let texture_format = context
            .surface_texture_format
            .unwrap_or(Shader::TEXTURE_FORMAT);
        let sample_count = anti_aliasing.map_or(1, |a| a.mode.sample_count());
        Self {
            size,
            color_buffer_view: Self::create_color_buffer_view(
                size,
                sample_count,
                texture_format,
                context,
            ),
            depth_buffer_view: Self::create_depth_buffer_view(size, sample_count, context),
            encoder: None,
            texture: None,
            texture_format,
            sample_count,
        }
    }

    pub(crate) fn size(&self) -> NonZeroSize {
        self.size
    }

    pub(crate) fn update(
        &mut self,
        size: NonZeroSize,
        anti_aliasing: Option<&AntiAliasing>,
        context: &GpuContext,
    ) {
        let sample_count = anti_aliasing.map_or(1, |a| a.mode.sample_count());
        let texture_format = context
            .surface_texture_format
            .unwrap_or(Shader::TEXTURE_FORMAT);
        if self.size != size
            || self.sample_count != sample_count
            || self.texture_format != texture_format
        {
            self.size = size;
            self.sample_count = sample_count;
            self.color_buffer_view = Self::create_color_buffer_view(
                self.size,
                sample_count,
                self.texture_format,
                context,
            );
            self.depth_buffer_view =
                Self::create_depth_buffer_view(self.size, sample_count, context);
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
        let view = self.texture.insert(view);
        self.encoder
            .insert(context.device.create_command_encoder(&descriptor))
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("modor_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: if self.sample_count > 1 {
                        &self.color_buffer_view
                    } else {
                        view
                    },
                    resolve_target: (self.sample_count > 1).then_some(view),
                    ops: Operations {
                        load: LoadOp::Clear(background_color.into()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            })
    }

    pub(crate) fn submit_command_queue(&mut self, context: &GpuContext) {
        let encoder = self
            .encoder
            .take()
            .expect("internal error: encoder not initialized");
        context.queue.submit(Some(encoder.finish()));
    }

    fn create_color_buffer_view(
        size: NonZeroSize,
        sample_count: u32,
        texture_format: TextureFormat,
        context: &GpuContext,
    ) -> TextureView {
        let texture = context.device.create_texture(&TextureDescriptor {
            label: Some("modor_color_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: texture_format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }

    fn create_depth_buffer_view(
        size: NonZeroSize,
        sample_count: u32,
        context: &GpuContext,
    ) -> TextureView {
        let texture = context.device.create_texture(&TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width: size.width.into(),
                height: size.height.into(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&TextureViewDescriptor::default())
    }
}
