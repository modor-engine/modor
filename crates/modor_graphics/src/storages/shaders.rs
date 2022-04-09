use crate::backend::data::{GpuData, Instance, Vertex};
use crate::backend::renderer::Renderer;
use crate::backend::shaders::Shader;
use modor_physics::Shape;
use typed_index_collections::TiVec;

pub(super) struct ShaderStorage {
    shaders: TiVec<ShaderIdx, Shader>,
}

impl ShaderStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        let vertex_buffer_layouts = &[
            <Vertex as GpuData<0>>::layout(),
            <Instance as GpuData<1>>::layout(),
        ];
        Self {
            shaders: ti_vec![
                Shader::new(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
                    vertex_buffer_layouts,
                    "main_2d",
                    &renderer,
                ),
                Shader::new(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/circle.wgsl")),
                    vertex_buffer_layouts,
                    "circle_2d",
                    &renderer,
                )
            ],
        }
    }

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