use crate::buffer::Buffer;
use crate::gpu::{Gpu, GpuHandle, GpuState};
use crate::vertex_buffer::VertexBuffer;
use modor::{Context, Glob, GlobRef, NoVisit, Node};
use wgpu::{vertex_attr_array, BufferUsages, VertexAttribute, VertexStepMode};

#[derive(NoVisit, Debug)]
pub(crate) struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    label: String,
    glob: Glob<Option<MeshGlob>>,
    gpu: GpuHandle,
}

impl Node for Mesh {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        match self.gpu.get(ctx) {
            GpuState::None => {
                *self.glob.get_mut(ctx) = None;
            }
            GpuState::New(gpu) => {
                *self.glob.get_mut(ctx) = Some(MeshGlob::new(
                    &gpu,
                    &self.vertices,
                    &self.indices,
                    &self.label,
                ));
            }
            GpuState::Same(_) => (),
        };
    }
}

impl Mesh {
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

    pub(crate) fn glob(&self) -> &GlobRef<Option<MeshGlob>> {
        self.glob.as_ref()
    }

    fn new(
        ctx: &mut Context<'_>,
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        label: impl Into<String>,
    ) -> Self {
        Self {
            vertices,
            indices,
            label: label.into(),
            glob: Glob::new(ctx, None),
            gpu: GpuHandle::default(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct MeshGlob {
    pub(crate) index_count: usize,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<u16>,
    gpu_version: u64,
}

impl MeshGlob {
    pub(crate) fn vertices(&self, gpu: &Gpu) -> Option<&Buffer<Vertex>> {
        (gpu.version == self.gpu_version).then_some(&self.vertex_buffer)
    }

    pub(crate) fn indices(&self, gpu: &Gpu) -> Option<&Buffer<u16>> {
        (gpu.version == self.gpu_version).then_some(&self.index_buffer)
    }

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
            gpu_version: gpu.version,
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
