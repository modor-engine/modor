use crate::buffer::Buffer;
use crate::gpu::GpuManager;
use modor::{App, FromApp, Global};
use wgpu::{
    vertex_attr_array, BufferAddress, BufferUsages, VertexAttribute, VertexBufferLayout,
    VertexStepMode,
};

#[derive(Debug, Global)]
pub(crate) struct Mesh {
    pub(crate) vertex_buffer: Buffer<Vertex>,
    pub(crate) index_buffer: Buffer<u16>,
}

impl FromApp for Mesh {
    fn from_app(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init();
        let vertices = &[
            Vertex {
                position: [-0.5, 0.5, 0.],
                texture_position: [0., 0.],
            },
            Vertex {
                position: [-0.5, -0.5, 0.],
                texture_position: [0., 1.],
            },
            Vertex {
                position: [0.5, -0.5, 0.],
                texture_position: [1., 1.],
            },
            Vertex {
                position: [0.5, 0.5, 0.],
                texture_position: [1., 0.],
            },
        ];
        let indices = &[0, 1, 2, 0, 2, 3];
        Self {
            vertex_buffer: Buffer::new(gpu, vertices, BufferUsages::VERTEX, "mesh_vertices"),
            index_buffer: Buffer::new(gpu, indices, BufferUsages::INDEX, "mesh_indices"),
        }
    }
}

pub(crate) trait VertexBuffer<const L: u32>: Sized {
    const ATTRIBUTES: &'static [VertexAttribute];
    const STEP_MODE: VertexStepMode;
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: Self::STEP_MODE,
        attributes: <Self as VertexBuffer<L>>::ATTRIBUTES,
    };
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Vertex {
    position: [f32; 3],
    texture_position: [f32; 2],
}

impl<const L: u32> VertexBuffer<L> for Vertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![L => Float32x3, L + 1 => Float32x2];
    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}
