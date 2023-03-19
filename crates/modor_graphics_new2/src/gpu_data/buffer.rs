use crate::Renderer;
use bytemuck::Pod;
use std::fmt::Debug;
use std::mem;
use std::ops::{Deref, DerefMut};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BindingResource, Buffer, BufferDescriptor, BufferSlice, BufferUsages, Device};

#[derive(Debug)]
pub(crate) struct DynamicBuffer<T>
where
    T: Debug,
{
    data: Vec<T>,
    is_new: bool,
    is_changed: bool,
    buffer: Buffer,
    usage: DynamicBufferUsage,
    label: String,
    buffer_capacity: usize,
}

impl<T> DynamicBuffer<T>
where
    T: Pod + Sync + Send + Debug,
{
    pub(crate) fn new(
        data: Vec<T>,
        usage: DynamicBufferUsage,
        label: impl Into<String>,
        device: &Device,
    ) -> Self {
        let label = label.into();
        Self {
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&label),
                contents: bytemuck::cast_slice(&data),
                usage: usage.into(),
            }),
            is_new: true,
            is_changed: false,
            usage,
            buffer_capacity: data.capacity(),
            data,
            label,
        }
    }

    pub(crate) fn is_changed(&self) -> bool {
        self.is_new || self.is_changed
    }

    pub(crate) fn resource(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    pub(crate) fn buffer(&self) -> BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub(crate) fn sync(&mut self, renderer: &Renderer) {
        self.is_new = false;
        if !self.is_changed {
            return;
        }
        self.is_changed = false;
        if self.buffer_capacity != self.data.capacity() {
            self.buffer_capacity = self.data.capacity();
            self.buffer = renderer.device.create_buffer(&BufferDescriptor {
                label: Some(&self.label),
                size: Self::raw_capacity(self.data.capacity()),
                usage: self.usage.into(),
                mapped_at_creation: false,
            });
        }
        renderer
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.data));
    }

    fn raw_capacity(capacity: usize) -> u64 {
        let raw_capacity = (capacity * mem::size_of::<T>()) as u64;
        nearest_multiple(raw_capacity, wgpu::COPY_BUFFER_ALIGNMENT)
    }
}

impl<T> Deref for DynamicBuffer<T>
where
    T: Debug,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for DynamicBuffer<T>
where
    T: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.is_changed = true;
        &mut self.data
    }
}

#[derive(Debug, Clone, Copy)]
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
