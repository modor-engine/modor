use crate::resources::buffers::{DynamicBuffer, DynamicBufferUsage, GpuData};
use fxhash::FxHashMap;
use modor::{Built, Changed, Entity, EntityBuilder, Filter, Query};
use modor_internal::dyn_types::DynType;
use wgpu::{vertex_attr_array, Device, VertexAttribute, VertexStepMode};

pub(crate) struct ModelRegistry {
    entity_ids: FxHashMap<ModelKey, usize>,
}

#[singleton]
impl ModelRegistry {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            entity_ids: FxHashMap::default(),
        })
    }

    #[run]
    fn register(&mut self, models: Query<'_, (&Model, Entity<'_>, Filter<Changed<Model>>)>) {
        for (model, entity, _) in models.iter() {
            self.entity_ids.insert(model.key.clone(), entity.id());
        }
    }

    pub(crate) fn find<'a>(
        &self,
        key: &ModelKey,
        query: &'a Query<'_, &Model>,
    ) -> Option<&'a Model> {
        self.entity_ids.get(key).and_then(|&i| query.get(i))
    }
}

pub(crate) struct Model {
    key: ModelKey,
    vertex_buffer: DynamicBuffer<Vertex>,
    index_buffer: DynamicBuffer<u16>,
}

#[entity]
impl Model {
    pub(crate) fn build_rectangle(device: &Device) -> impl Built<Self> {
        Self::build(
            RECTANGLE_VERTICES.to_vec(),
            RECTANGLE_INDICES.to_vec(),
            ModelKey::new(ModelRef::Rectangle),
            device,
        )
    }

    fn build(
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        key: ModelKey,
        device: &Device,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self {
            key: key.clone(),
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
        })
    }

    pub(crate) fn vertex_buffer(&self) -> &DynamicBuffer<Vertex> {
        &self.vertex_buffer
    }

    pub(crate) fn index_buffer(&self) -> &DynamicBuffer<u16> {
        &self.index_buffer
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ModelKey(DynType);

impl ModelKey {
    pub(crate) fn new(ref_: ModelRef) -> Self {
        Self(DynType::new(ref_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ModelRef {
    Rectangle,
}

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

const RECTANGLE_VERTICES: [Vertex; 4] = [
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
const RECTANGLE_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];
