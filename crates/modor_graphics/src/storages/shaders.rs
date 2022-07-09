use crate::backend::data::{Camera, GpuData, Instance, Vertex};
use crate::backend::renderer::Renderer;
use crate::backend::shaders::Shader;
use crate::backend::uniforms::Uniform;
use crate::{Mesh, Shape};
use typed_index_collections::TiVec;

const FIRST_VERTEX_BUFFER_LOCATION: u32 = 0;
const FIRST_INSTANCE_BUFFER_LOCATION: u32 = 1;

pub(super) struct ShaderStorage {
    shaders: TiVec<ShaderIdx, Shader>,
}

impl ShaderStorage {
    pub(super) fn new(camera_2d: &Uniform<Camera>, renderer: &Renderer) -> Self {
        let vertex_buffer_layouts = &[
            <Vertex as GpuData<FIRST_VERTEX_BUFFER_LOCATION>>::layout(),
            <Instance as GpuData<FIRST_INSTANCE_BUFFER_LOCATION>>::layout(),
        ];
        Self {
            shaders: ti_vec![
                Shader::new(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
                    vertex_buffer_layouts,
                    &[camera_2d.bind_group_layout()],
                    "main_2d",
                    renderer,
                ),
                Shader::new(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
                    vertex_buffer_layouts,
                    &[camera_2d.bind_group_layout()],
                    "ellipse_2d",
                    renderer,
                )
            ],
        }
    }

    pub(super) fn idx(mesh: &Mesh) -> ShaderIdx {
        match mesh.shape {
            Shape::Rectangle => ShaderIdx(0),
            Shape::Ellipse => ShaderIdx(1),
        }
    }

    pub(super) fn get(&self, idx: ShaderIdx) -> &Shader {
        &self.shaders[idx]
    }
}

idx_type!(pub(super) ShaderIdx);
