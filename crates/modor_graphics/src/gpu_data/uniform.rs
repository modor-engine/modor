use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::GpuContext;
use bytemuck::Pod;
use modor::{VariableSend, VariableSync};
use std::fmt::Debug;
use wgpu::{BindGroup, BindGroupLayout, BindingResource, Device};

#[derive(Debug)]
pub(crate) struct Uniform<T>
where
    T: Debug,
{
    buffer: DynamicBuffer<T>,
    bind_group: BindGroup,
}

impl<T> Uniform<T>
where
    T: Pod + VariableSync + VariableSend + Debug,
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
            format!("modor_uniform_buffer_{label_suffix}"),
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

    pub(crate) fn get_mut(&mut self) -> &mut T {
        &mut self.buffer[0]
    }

    pub(crate) fn is_changed(&self) -> bool {
        self.buffer.is_changed()
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub(crate) fn sync(&mut self, context: &GpuContext) {
        self.buffer.sync(context);
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
            label: Some(&format!("modor_uniform_bind_group_{label_suffix}")),
        })
    }
}
