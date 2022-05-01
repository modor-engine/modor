use crate::backend::data::{GpuData, Instance, Vertex};
use crate::backend::renderer::Renderer;
use crate::backend::shaders::Shader;
use modor_physics::Shape;
use typed_index_collections::TiVec;

const FIRST_VERTEX_BUFFER_LOCATION: u32 = 0;
const FIRST_INSTANCE_BUFFER_LOCATION: u32 = 1;

pub(super) struct ShaderStorage {
    shaders: TiVec<ShaderIdx, Shader>,
}

impl ShaderStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        let vertex_buffer_layouts = &[
            <Vertex as GpuData<FIRST_VERTEX_BUFFER_LOCATION>>::layout(),
            <Instance as GpuData<FIRST_INSTANCE_BUFFER_LOCATION>>::layout(),
        ];
        Self {
            shaders: ti_vec![
                Shader::new(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
                    vertex_buffer_layouts,
                    "main_2d",
                    renderer,
                ),
                Shader::new(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/circle.wgsl")),
                    vertex_buffer_layouts,
                    "circle_2d",
                    renderer,
                )
            ],
        }
    }

    #[allow(clippy::unused_self)] // will be used in the future
    pub(super) fn idx(&self, shape: &Shape) -> ShaderIdx {
        match shape {
            Shape::Rectangle2D => ShaderIdx(0),
            Shape::Circle2D => ShaderIdx(1),
        }
    }

    pub(super) fn get(&self, idx: ShaderIdx) -> &Shader {
        &self.shaders[idx]
    }
}

idx_type!(pub(super) ShaderIdx);
