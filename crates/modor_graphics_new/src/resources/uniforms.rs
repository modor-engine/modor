use crate::resources::buffers::{DynamicBuffer, DynamicBufferUsage};
use crate::targets::GpuDevice;
use bytemuck::Pod;
use std::ops::{Deref, DerefMut};
use wgpu::{BindGroup, BindGroupLayout, BindingResource, Device, RenderPass};

pub(crate) struct Uniform<T> {
    buffer: DynamicBuffer<T>,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl<T> Uniform<T>
where
    T: Pod + Sync + Send,
{
    pub(crate) fn new(value: T, binding: u32, label_suffix: &str, device: &Device) -> Self {
        let buffer = DynamicBuffer::new(
            vec![value],
            DynamicBufferUsage::Uniform,
            format!("modor_uniform_buffer_{}", label_suffix),
            device,
        );
        let bind_group_layout = Self::create_bind_group_layout(binding, label_suffix, device);
        Self {
            bind_group: Self::create_bind_group(
                binding,
                label_suffix,
                device,
                buffer.resource(),
                &bind_group_layout,
            ),
            buffer,
            bind_group_layout,
        }
    }

    pub(crate) fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub(crate) fn sync(&mut self, device: &GpuDevice) {
        self.buffer.sync(device);
    }

    pub(crate) fn use_for_rendering<'a>(&'a self, group: u32, pass: &mut RenderPass<'a>) {
        pass.set_bind_group(group, &self.bind_group, &[]);
    }

    fn create_bind_group_layout(
        binding: u32,
        label_suffix: &str,
        device: &Device,
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some(&format!("modor_uniform_bind_group_layout_{}", label_suffix)),
        })
    }

    fn create_bind_group(
        binding: u32,
        label_suffix: &str,
        device: &Device,
        resource: BindingResource<'_>,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding, resource }],
            label: Some(&format!("modor_uniform_bind_group_{}", label_suffix)),
        })
    }
}

impl<T> Deref for Uniform<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.buffer[0]
    }
}

impl<T> DerefMut for Uniform<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer[0]
    }
}
