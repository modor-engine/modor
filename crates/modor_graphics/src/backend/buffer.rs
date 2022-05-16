use crate::backend::renderer::Renderer;
use crate::utils;
use bytemuck::Pod;
use std::mem;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindingResource, Buffer, BufferDescriptor, BufferUsages};

pub(crate) struct DynamicBuffer<T> {
    data: Vec<T>,
    usage: DynamicBufferUsage,
    label: String,
    buffer: Buffer,
    buffer_capacity: usize,
}

impl<T> DynamicBuffer<T>
where
    T: Pod,
{
    pub(crate) fn new(
        data: Vec<T>,
        usage: DynamicBufferUsage,
        label: String,
        renderer: &Renderer,
    ) -> Self {
        Self {
            usage,
            buffer: renderer.device().create_buffer_init(&BufferInitDescriptor {
                label: Some(&label),
                contents: bytemuck::cast_slice(&data),
                usage: usage.into(),
            }),
            buffer_capacity: data.capacity(),
            data,
            label,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub(crate) fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub(crate) fn sync(&mut self, renderer: &Renderer) {
        if self.buffer_capacity != self.data.capacity() {
            self.buffer_capacity = self.data.capacity();
            self.buffer = renderer.device().create_buffer(&BufferDescriptor {
                label: Some(&self.label),
                size: Self::raw_capacity(self.data.capacity()),
                usage: self.usage.into(),
                mapped_at_creation: false,
            });
        }
        renderer
            .queue()
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    pub(super) fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub(super) fn binding_resource(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    fn raw_capacity(capacity: usize) -> u64 {
        let raw_capacity = (capacity * mem::size_of::<T>()) as u64;
        utils::nearest_multiple(raw_capacity, wgpu::COPY_BUFFER_ALIGNMENT)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DynamicBufferUsage {
    Vertex,
    Index,
    Instance,
    Uniform,
}

impl From<DynamicBufferUsage> for BufferUsages {
    fn from(usage: DynamicBufferUsage) -> Self {
        match usage {
            DynamicBufferUsage::Vertex => Self::VERTEX,
            DynamicBufferUsage::Index => Self::INDEX,
            DynamicBufferUsage::Instance => Self::VERTEX | Self::COPY_DST,
            DynamicBufferUsage::Uniform => Self::UNIFORM | Self::COPY_DST,
        }
    }
}
