use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Vertex;
use crate::backend::renderer::Renderer;
use modor_physics::Shape;
use typed_index_collections::TiVec;

const RECTANGLE_VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-0.5, 0.5, 0.],
    },
    Vertex {
        position: [-0.5, -0.5, 0.],
    },
    Vertex {
        position: [0.5, -0.5, 0.],
    },
    Vertex {
        position: [0.5, 0.5, 0.],
    },
];
const RECTANGLE_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

pub(super) struct ModelStorage {
    models: TiVec<ModelIdx, Model>,
}

impl ModelStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        Self {
            models: ti_vec![Model::new(
                RECTANGLE_VERTICES.to_vec(),
                RECTANGLE_INDICES.to_vec(),
                "rectangle_2d",
                renderer,
            )],
        }
    }

    #[allow(clippy::unused_self)] // will be used in the future
    pub(super) fn idx(&self, shape: &Shape) -> ModelIdx {
        match shape {
            Shape::Rectangle2D | Shape::Circle2D => ModelIdx(0),
        }
    }

    pub(super) fn get(&self, idx: ModelIdx) -> &Model {
        &self.models[idx]
    }
}

idx_type!(pub(super) ModelIdx);

pub(super) struct Model {
    pub(super) vertex_buffer: DynamicBuffer<Vertex>,
    pub(super) index_buffer: DynamicBuffer<u16>,
}

impl Model {
    fn new(vertices: Vec<Vertex>, indices: Vec<u16>, label: &str, renderer: &Renderer) -> Self {
        Self {
            vertex_buffer: DynamicBuffer::new(
                vertices,
                DynamicBufferUsage::Vertex,
                format!("modor_vertex_buffer_{}", label),
                renderer,
            ),
            index_buffer: DynamicBuffer::new(
                indices,
                DynamicBufferUsage::Index,
                format!("modor_index_buffer_{}", label),
                renderer,
            ),
        }
    }
}
