use crate::resources::buffers::{DynamicBuffer, DynamicBufferUsage, GpuData};
use crate::resources::registries::{Resource, ResourceRegistry};
use crate::ResourceKey;
use modor::{Built, EntityBuilder};
use modor_internal::dyn_types::DynType;
use wgpu::{vertex_attr_array, Device, VertexAttribute, VertexStepMode};

pub(crate) type ModelRegistry = ResourceRegistry<Model>;

pub(crate) struct Model {
    key: DynType,
    vertex_buffer: DynamicBuffer<Vertex>,
    index_buffer: DynamicBuffer<u16>,
}

#[component]
impl Model {
    fn new(
        key: impl ResourceKey,
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        device: &Device,
    ) -> Self {
        Self {
            vertex_buffer: DynamicBuffer::new(
                vertices,
                DynamicBufferUsage::Vertex,
                format!("modor_vertex_buffer_{:?}", key),
                device,
            ),
            index_buffer: DynamicBuffer::new(
                indices,
                DynamicBufferUsage::Index,
                format!("modor_index_buffer_{:?}", key),
                device,
            ),
            key: DynType::new(key),
        }
    }

    pub(crate) fn vertex_buffer(&self) -> &DynamicBuffer<Vertex> {
        &self.vertex_buffer
    }

    pub(crate) fn index_buffer(&self) -> &DynamicBuffer<u16> {
        &self.index_buffer
    }
}

impl Resource for Model {
    fn key(&self) -> &DynType {
        &self.key
    }
}

pub(crate) struct RectangleModel;

#[singleton]
impl RectangleModel {
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

    pub(crate) fn build(device: &Device) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Model::new(
            RectangleModelKey,
            Self::VERTICES.to_vec(),
            Self::INDICES.to_vec(),
            device,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RectangleModelKey;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) texture_position: [f32; 2],
}

impl<const L: u32> GpuData<L> for Vertex {
    const ATTRIBUTES: &'static [VertexAttribute] =
        &vertex_attr_array![L => Float32x3, L + 1 => Float32x2];
    const STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}
