use crate::backend::buffer::DynamicBuffer;
use crate::backend::renderer::Renderer;
use crate::backend::shaders::Shader;
use crate::backend::textures::Texture;
use crate::backend::uniforms::Uniform;
use bytemuck::Pod;
use std::ops::Range;
use wgpu::{
    Color, CommandEncoder, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureView,
};

pub(crate) struct Rendering<'a> {
    renderer: &'a mut Renderer,
    surface: TextureView,
    encoder: CommandEncoder,
}

impl<'a> Rendering<'a> {
    pub(crate) fn new(renderer: &'a mut Renderer) -> Self {
        let surface = renderer.prepare_texture();
        let encoder = renderer
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("modor_render_encoder"),
            });
        Self {
            renderer,
            surface,
            encoder,
        }
    }

    pub(crate) fn apply(self) {
        self.renderer.render(self.encoder);
    }
}

pub(crate) struct RenderCommands<'a> {
    pass: RenderPass<'a>,
}

impl<'a> RenderCommands<'a> {
    pub(crate) fn new(background_color: Color, rendering: &'a mut Rendering<'_>) -> Self {
        let pass = rendering.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("modor_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &rendering.surface,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(background_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: rendering.renderer.depth_buffer(),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        Self { pass }
    }

    pub(crate) fn push_uniform_binding<T>(&mut self, uniform: &'a Uniform<T>, group: u32)
    where
        T: Pod,
    {
        self.pass.set_bind_group(group, uniform.bind_group(), &[]);
    }

    pub(crate) fn push_texture_binding(&mut self, texture: &'a Texture, group: u32) {
        self.pass.set_bind_group(group, texture.bind_group(), &[]);
    }

    pub(crate) fn push_shader_change(&mut self, shader: &'a Shader) {
        self.pass.set_pipeline(shader.pipeline());
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn push_draw<V, I>(
        &mut self,
        vertex_buffer: &'a DynamicBuffer<V>,
        index_buffer: &'a DynamicBuffer<u16>,
        instance_buffer: &'a DynamicBuffer<I>,
        drawn_instance_idxs: Range<usize>,
    ) where
        V: Pod,
        I: Pod,
    {
        self.pass
            .set_vertex_buffer(0, vertex_buffer.buffer().slice(..));
        self.pass
            .set_vertex_buffer(1, instance_buffer.buffer().slice(..));
        self.pass
            .set_index_buffer(index_buffer.buffer().slice(..), IndexFormat::Uint16);
        self.pass.draw_indexed(
            0..(index_buffer.len() as u32),
            0,
            (drawn_instance_idxs.start as u32)..(drawn_instance_idxs.end as u32),
        );
    }
}
