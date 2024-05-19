use crate::buffer::Buffer;
use crate::gpu::{Gpu, GpuManager};
use crate::vertex_buffer::VertexBuffer;
use modor::{Context, Glob, GlobRef, Node, Visit};
use wgpu::{vertex_attr_array, BufferUsages, VertexAttribute, VertexStepMode};

#[derive(Debug, Visit, Node)]
pub(crate) struct Mesh {
    label: String,
    glob: Glob<MeshGlob>,
}

impl Mesh {
    fn new(
        ctx: &mut Context<'_>,
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        label: impl Into<String>,
    ) -> Self {
        let label = label.into();
        let gpu = ctx.get_mut::<GpuManager>().get();
        let glob = MeshGlob::new(gpu, &vertices, &indices, &label);
        Self {
            label,
            glob: Glob::new(ctx, glob),
        }
    }

    pub(crate) fn rectangle(ctx: &mut Context<'_>) -> Self {
        Self::new(
            ctx,
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
            "rectangle",
        )
    }

    pub(crate) fn glob(&self) -> &GlobRef<MeshGlob> {
        self.glob.as_ref()
    }
}

#[derive(Debug)]
pub(crate) struct MeshGlob {
    pub(crate) index_count: usize,
    pub(crate) vertex_buffer: Buffer<Vertex>,
    pub(crate) index_buffer: Buffer<u16>,
}

impl MeshGlob {
    fn new(gpu: &Gpu, vertices: &[Vertex], indices: &[u16], label: &str) -> Self {
        Self {
            index_count: indices.len(),
            vertex_buffer: Buffer::new(
                gpu,
                vertices,
                BufferUsages::VERTEX,
                format!("mesh_vertices:{label}"),
            ),
            index_buffer: Buffer::new(
                gpu,
                indices,
                BufferUsages::INDEX,
                format!("mesh_indices:{label}"),
            ),
        }
    }
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
