use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{Model, ResourceKey, ZIndex2D};
use modor::{Changed, Entity, Or};
use modor_math::{Mat4, Quat};
use modor_physics::Transform2D;
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

pub(crate) mod opaque;
pub(crate) mod transparent;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GroupKey {
    pub(crate) camera_key: ResourceKey,
    pub(crate) material_key: ResourceKey,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    transform: [[f32; 4]; 4],
}

impl<const L: u32> VertexBuffer<L> for Instance {
    const ATTRIBUTES: &'static [VertexAttribute] = &vertex_attr_array![
        L => Float32x4,
        L + 1 => Float32x4,
        L + 2 => Float32x4,
        L + 3 => Float32x4,
    ];
    const STEP_MODE: VertexStepMode = VertexStepMode::Instance;
}

type Model2D<'a> = (&'a Transform2D, &'a Model, Option<&'a ZIndex2D>, Entity<'a>);
type ChangedModel2D = Or<(Changed<Transform2D>, Changed<Model>, Changed<ZIndex2D>)>;

// offset should be between 0. and 0.5
fn create_instance(transform: &Transform2D, z_index: Option<&ZIndex2D>, offset: f32) -> Instance {
    let z = z_index.cloned().unwrap_or_default().to_f32(offset);
    Instance {
        transform: (Mat4::from_scale(transform.size.with_z(0.))
            * Quat::from_z(*transform.rotation).matrix()
            * Mat4::from_position(transform.position.with_z(z)))
        .to_array(),
    }
}
