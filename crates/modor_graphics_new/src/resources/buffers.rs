use crate::targets::GpuDevice;
use bytemuck::Pod;
use std::mem;
use std::ops::{Deref, DerefMut};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindingResource, Buffer, BufferAddress, BufferDescriptor, BufferSlice, BufferUsages, Device,
    VertexAttribute, VertexBufferLayout, VertexStepMode,
};

pub(crate) struct DynamicBuffer<T> {
    data: Vec<T>,
    is_changed: bool,
    buffer: Buffer,
    usage: DynamicBufferUsage,
    label: String,
    buffer_capacity: usize,
}

impl<T> DynamicBuffer<T>
where
    T: Pod + Sync + Send,
{
    pub(crate) fn new(
        data: Vec<T>,
        usage: DynamicBufferUsage,
        label: String,
        device: &Device,
    ) -> Self {
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&label),
                contents: bytemuck::cast_slice(&data),
                usage: usage.into(),
            }),
            is_changed: false,
            usage,
            buffer_capacity: data.capacity(),
            data,
            label,
        }
    }

    pub(crate) fn resource(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    pub(crate) fn buffer(&self) -> BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub(crate) fn sync(&mut self, device: &GpuDevice) {
        if !self.is_changed {
            return;
        }
        if self.buffer_capacity != self.data.capacity() {
            self.buffer_capacity = self.data.capacity();
            self.buffer = device.device.create_buffer(&BufferDescriptor {
                label: Some(&self.label),
                size: Self::raw_capacity(self.data.capacity()),
                usage: self.usage.into(),
                mapped_at_creation: false,
            });
        }
        device
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    fn raw_capacity(capacity: usize) -> u64 {
        let raw_capacity = (capacity * mem::size_of::<T>()) as u64;
        nearest_multiple(raw_capacity, wgpu::COPY_BUFFER_ALIGNMENT)
    }
}

impl<T> Deref for DynamicBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for DynamicBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_changed = true;
        &mut self.data
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

pub(crate) trait GpuData<const L: u32>: Sized {
    const ATTRIBUTES: &'static [VertexAttribute];
    const STEP_MODE: VertexStepMode;
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as BufferAddress,
        step_mode: Self::STEP_MODE,
        attributes: <Self as GpuData<L>>::ATTRIBUTES,
    };
}

fn nearest_multiple(value: u64, multiple: u64) -> u64 {
    let align_mask = multiple - 1;
    (value + align_mask) & !align_mask
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod buffers_test {
    #[test]
    fn calculate_nearest_multiple() {
        assert_eq!(super::nearest_multiple(0, 4), 0);
        assert_eq!(super::nearest_multiple(1, 4), 4);
        assert_eq!(super::nearest_multiple(4, 4), 4);
        assert_eq!(super::nearest_multiple(5, 4), 8);
    }
}
