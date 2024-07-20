use crate::gpu::Gpu;
use bytemuck::NoUninit;
use std::marker::PhantomData;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferSlice,
    BufferUsages,
};

#[derive(Debug)]
pub(crate) struct Buffer<T> {
    inner: wgpu::Buffer,
    len: usize,
    usages: BufferUsages,
    label: String,
    phantom: PhantomData<fn(T)>,
}

impl<T> Buffer<T>
where
    T: NoUninit + PartialEq,
{
    pub(crate) fn new(
        gpu: &Gpu,
        data: &[T],
        usages: BufferUsages,
        label: impl Into<String>,
    ) -> Self {
        let data = bytemuck::cast_slice(data);
        let label = label.into();
        Self {
            inner: Self::create_buffer(gpu, data, usages, &label),
            len: data.len(),
            usages,
            label,
            phantom: PhantomData,
        }
    }

    pub(crate) fn resource(&self) -> BindingResource<'_> {
        self.inner.as_entire_binding()
    }

    pub(crate) fn slice(&self) -> BufferSlice<'_> {
        self.inner.slice(..)
    }

    pub(crate) fn update(&mut self, gpu: &Gpu, data: &[T]) {
        let data = bytemuck::cast_slice(data);
        if self.len < data.len() {
            self.inner = Self::create_buffer(gpu, data, self.usages, &self.label);
            self.len = data.len();
        } else {
            gpu.queue.write_buffer(&self.inner, 0, data);
        }
    }

    fn create_buffer(gpu: &Gpu, data: &[u8], usages: BufferUsages, label: &str) -> wgpu::Buffer {
        gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(&format!("modor_buffer:{label}")),
            contents: data,
            usage: usages,
        })
    }
}

#[derive(Debug)]
pub(crate) struct BufferBindGroup {
    pub(crate) inner: BindGroup,
}

impl BufferBindGroup {
    pub(crate) fn new(
        gpu: &Gpu,
        entries: &[BindGroupEntry<'_>],
        layout: &BindGroupLayout,
        label: &str,
    ) -> Self {
        Self {
            inner: gpu.device.create_bind_group(&BindGroupDescriptor {
                layout,
                entries,
                label: Some(&format!("modor_bind_group:{label}")),
            }),
        }
    }

    pub(crate) fn uniform<T>(
        gpu: &Gpu,
        buffer: &Buffer<T>,
        binding: u32,
        layout: &BindGroupLayout,
        label: &str,
    ) -> Self
    where
        T: NoUninit + PartialEq,
    {
        Self::new(
            gpu,
            &[BindGroupEntry {
                binding,
                resource: buffer.resource(),
            }],
            layout,
            label,
        )
    }
}
