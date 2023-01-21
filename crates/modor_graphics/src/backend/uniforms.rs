use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::renderer::Renderer;
use bytemuck::Pod;
use wgpu::{BindGroup, BindGroupLayout};

pub(crate) struct Uniform<T> {
    buffer: DynamicBuffer<T>,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl<T> Uniform<T>
where
    T: Pod,
{
    pub(crate) fn new(data: Vec<T>, binding: u32, label_suffix: &str, renderer: &Renderer) -> Self {
        let buffer = DynamicBuffer::new(
            data,
            DynamicBufferUsage::Uniform,
            format!("modor_uniform_buffer_{label_suffix}"),
            renderer,
        );
        let bind_group_layout = Self::create_bind_group_layout(binding, label_suffix, renderer);
        let bind_group =
            Self::create_bind_group(binding, label_suffix, renderer, &buffer, &bind_group_layout);
        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub(crate) fn buffer_mut(&mut self) -> &mut DynamicBuffer<T> {
        &mut self.buffer
    }

    pub(crate) fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub(super) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn create_bind_group_layout(
        binding: u32,
        label_suffix: &str,
        renderer: &Renderer,
    ) -> BindGroupLayout {
        renderer
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some(&format!("modor_uniform_bind_group_layout_{label_suffix}")),
            })
    }

    fn create_bind_group(
        binding: u32,
        label_suffix: &str,
        renderer: &Renderer,
        buffer: &DynamicBuffer<T>,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        renderer
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding,
                    resource: buffer.binding_resource(),
                }],
                label: Some(&format!("modor_uniform_bind_group_{label_suffix}")),
            })
    }
}
