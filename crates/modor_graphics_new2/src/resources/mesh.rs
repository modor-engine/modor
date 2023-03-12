use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{
    GraphicsModule, IntoResourceKey, Resource, ResourceKey, ResourceRegistry, ResourceState,
};
use modor::{Built, EntityBuilder, Single};
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

pub(crate) type MeshRegistry = ResourceRegistry<Mesh>;

pub(crate) struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    key: ResourceKey,
    vertex_buffer: Option<DynamicBuffer<Vertex>>,
    index_buffer: Option<DynamicBuffer<u16>>,
}

#[component]
impl Mesh {
    fn from_memory(key: impl IntoResourceKey, vertices: Vec<Vertex>, indices: Vec<u16>) -> Self {
        Self {
            key: key.into_key(),
            vertices,
            indices,
            vertex_buffer: None,
            index_buffer: None,
        }
    }

    #[run]
    fn update(&mut self, module: Option<Single<'_, GraphicsModule>>) {
        if let Some(module) = module {
            if self.vertex_buffer.is_none() {
                self.vertex_buffer = Some(DynamicBuffer::new(
                    self.vertices.clone(),
                    DynamicBufferUsage::Vertex,
                    format!("modor_vertex_buffer_{:?}", self.key),
                    &module.device,
                ));
            }
            if self.index_buffer.is_none() {
                self.index_buffer = Some(DynamicBuffer::new(
                    self.indices.clone(),
                    DynamicBufferUsage::Index,
                    format!("modor_index_buffer_{:?}", self.key),
                    &module.device,
                ));
            }
        } else {
            self.vertex_buffer = None;
            self.index_buffer = None;
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

pub(crate) struct RectangleMesh;

#[singleton]
impl RectangleMesh {
    const VERTICES: [Vertex; 4] = [
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
    const INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).with(Mesh::from_memory(
            MeshKey::Rectangle,
            Self::VERTICES.to_vec(),
            Self::INDICES.to_vec(),
        ))
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