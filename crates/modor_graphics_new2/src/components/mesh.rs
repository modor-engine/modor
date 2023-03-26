use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{IntoResourceKey, Renderer, Resource, ResourceKey, ResourceRegistry, ResourceState};
use modor::Single;
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

pub(crate) type MeshRegistry = ResourceRegistry<Mesh>;

#[derive(Component, Debug)]
pub(crate) struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    key: ResourceKey,
    vertex_buffer: Option<DynamicBuffer<Vertex>>,
    index_buffer: Option<DynamicBuffer<u16>>,
    renderer_version: Option<u8>,
}

#[systems]
impl Mesh {
    pub(crate) fn rectangle() -> Self {
        Self::from_memory(
            MeshKey::Rectangle,
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

    fn from_memory(key: impl IntoResourceKey, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            key: key.into_key(),
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
            renderer_version: None,
        }
    }

    #[run_after(component(Renderer))]
    fn update(&mut self, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.vertex_buffer = None;
            self.index_buffer = None;
        }
        if let Some(context) = state.context() {
            if self.vertex_buffer.is_none() {
                self.vertex_buffer = Some(DynamicBuffer::new(
                    self.vertices.clone(),
                    DynamicBufferUsage::Vertex,
                    format!("modor_vertex_buffer_{:?}", self.key),
                    &context.device,
                ));
            }
            if self.index_buffer.is_none() {
                self.index_buffer = Some(DynamicBuffer::new(
                    self.indices.clone(),
                    DynamicBufferUsage::Index,
                    format!("modor_index_buffer_{:?}", self.key),
                    &context.device,
                ));
            }
        }
    }

    pub(crate) fn vertex_buffer(&self) -> &DynamicBuffer<Vertex> {
        self.vertex_buffer
            .as_ref()
            .expect("internal error: not initialized vertex buffer")
    }

    pub(crate) fn index_buffer(&self) -> &DynamicBuffer<u16> {
        self.index_buffer
            .as_ref()
            .expect("internal error: not initialized index buffer")
    }
}

impl Resource for Mesh {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.vertex_buffer.is_some() && self.index_buffer.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::NotLoaded
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MeshKey {
    Rectangle,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Vertex {
    position: [f32; 3],
    texture_position: [f32; 2],
}

impl<const L: u32> VertexBuffer<L> for Vertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![L => Float32x3, L + 1 => Float32x2];
    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}
