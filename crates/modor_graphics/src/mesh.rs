use crate::buffer::Buffer;
use crate::gpu::{Gpu, GpuManager};
use modor::{App, FromApp, Glob, Node};
use std::mem;
use wgpu::{
    vertex_attr_array, BufferAddress, BufferUsages, VertexAttribute, VertexBufferLayout,
    VertexStepMode,
};

#[derive(Debug, Node)]
pub(crate) struct Mesh {
    glob: Glob<MeshGlob>,
}

impl Mesh {
    fn new(app: &mut App, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let glob = Glob::<MeshGlob>::from_app(app);
        glob.get_mut(app).load(&gpu, &vertices, &indices);
        Self { glob }
    }

    pub(crate) fn rectangle(app: &mut App) -> Self {
        Self::new(
            app,
            vec![
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
            ],
            vec![0, 1, 2, 0, 2, 3],
        )
    }

    pub(crate) fn glob(&self) -> &Glob<MeshGlob> {
        &self.glob
    }
}

#[derive(Debug)]
pub(crate) struct MeshGlob {
    pub(crate) vertex_buffer: Buffer<Vertex>,
    pub(crate) index_buffer: Buffer<u16>,
}

impl FromApp for MeshGlob {
    fn from_app(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init();
        Self {
            vertex_buffer: Buffer::new(gpu, &[], BufferUsages::VERTEX, "mesh_vertices"),
            index_buffer: Buffer::new(gpu, &[], BufferUsages::INDEX, "mesh_indices"),
        }
    }
}

impl MeshGlob {
    fn load(&mut self, gpu: &Gpu, vertices: &[Vertex], indices: &[u16]) {
        self.vertex_buffer.update(gpu, vertices);
        self.index_buffer.update(gpu, indices);
    }
}

pub(crate) trait VertexBuffer<const L: u32>: Sized {
    const ATTRIBUTES: &'static [VertexAttribute];
    const STEP_MODE: VertexStepMode;
    const LAYOUT: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: mem::size_of::<Self>() as BufferAddress,
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
