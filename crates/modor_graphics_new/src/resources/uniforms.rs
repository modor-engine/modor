use crate::resources::buffers::{DynamicBuffer, DynamicBufferUsage};
use crate::targets::GpuDevice;
use bytemuck::Pod;
use std::ops::{Deref, DerefMut};
use wgpu::{BindGroup, BindGroupLayout, BindingResource, Device};

pub(crate) struct Uniform<T> {
    buffer: DynamicBuffer<T>,
    bind_group: BindGroup,
}

impl<T> Uniform<T>
where
    T: Pod + Sync + Send,
{
    pub(crate) fn new(
        value: T,
        binding: u32,
        bind_group_layout: &BindGroupLayout,
        label_suffix: &str,
        device: &Device,
    ) -> Self {
        let buffer = DynamicBuffer::new(
            vec![value],
            DynamicBufferUsage::Uniform,
            format!("modor_uniform_buffer_{}", label_suffix),
            device,
        );
        Self {
            bind_group: Self::create_bind_group(
                binding,
                label_suffix,
                device,
                buffer.resource(),
                bind_group_layout,
            ),
            buffer,
        }
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub(crate) fn sync(&mut self, device: &GpuDevice) {
        self.buffer.sync(device);
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
