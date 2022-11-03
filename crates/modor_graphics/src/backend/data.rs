use std::mem;
use wgpu::{vertex_attr_array, BufferAddress, VertexAttribute, VertexBufferLayout, VertexStepMode};

pub(crate) trait GpuData<const L: u32> {
    const ATTRIBUTES: &'static [VertexAttribute];

    fn layout() -> VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 3],
}

impl<const L: u32> GpuData<L> for Vertex {
    const ATTRIBUTES: &'static [VertexAttribute] = &vertex_attr_array![L => Float32x3];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: <Self as GpuData<L>>::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    pub(crate) transform: [[f32; 4]; 4],
    pub(crate) color: [f32; 4],
    pub(crate) has_texture: u32,
}

impl<const L: u32> GpuData<L> for Instance {
    const ATTRIBUTES: &'static [VertexAttribute] = &vertex_attr_array![
        L => Float32x4,
        L + 1 => Float32x4,
        L + 2 => Float32x4,
        L + 3 => Float32x4,
        L + 4 => Float32x4,
        L + 5 => Uint32,
    ];

    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: <Self as GpuData<L>>::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Camera {
    pub(crate) transform: [[f32; 4]; 4],
}
