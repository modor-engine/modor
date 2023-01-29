use crate::resources::buffers::GpuData;
use crate::resources::models::ModelKey;
use crate::resources::shaders::ShaderKey;
use crate::Mesh2D;
use modor_math::{Mat4, Quat};
use modor_physics::Transform2D;
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    pub(crate) transform: [[f32; 4]; 4],
    pub(crate) color: [f32; 4],
    pub(crate) has_texture: u32,
    pub(crate) texture_part_position: [f32; 2],
    pub(crate) texture_part_size: [f32; 2],
}

impl<const L: u32> GpuData<L> for Instance {
    const ATTRIBUTES: &'static [VertexAttribute] = &vertex_attr_array![
        L => Float32x4,
        L + 1 => Float32x4,
        L + 2 => Float32x4,
        L + 3 => Float32x4,
        L + 4 => Float32x4,
        L + 5 => Uint32,
        L + 6 => Float32x2,
        L + 7 => Float32x2,
    ];
    const STEP_MODE: VertexStepMode = VertexStepMode::Instance;
}

impl Instance {
    fn new(transform: &Transform2D, mesh: &Mesh2D) -> Self {
        let transform_matrix = Mat4::from_scale(transform.size.with_z(0.))
            * Quat::from_z(*transform.rotation).matrix()
            * Mat4::from_position(transform.position.with_z(mesh.z));
        Self {
            transform: transform_matrix.to_array(),
            color: mesh.color.into(),
            has_texture: false.into(),
            texture_part_position: [0., 0.],
            texture_part_size: [1., 1.],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ResourceKeys {
    pub(crate) shader: ShaderKey,
    pub(crate) model: ModelKey,
}

pub(crate) mod opaque;
