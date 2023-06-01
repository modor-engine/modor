use crate::components::material::MaterialRegistry;
use crate::components::renderer::Renderer;
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{Material, Model, ZIndex2D};
use modor::{Changed, Entity, Filter, Or, Query, Single, SingleMut};
use modor_math::{Mat4, Quat};
use modor_physics::Transform2D;
use modor_resources::ResourceKey;
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

// TODO: try to simplify sub modules

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GroupKey {
    pub(crate) camera_key: ResourceKey,
    pub(crate) material_key: ResourceKey,
    pub(crate) mesh_key: ResourceKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GroupKeyState {
    pub(crate) group_key: GroupKey,
    pub(crate) is_updated: bool,
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
type Model2DResources<'a, 'b, F> = (
    Single<'a, Renderer>,
    (SingleMut<'a, MaterialRegistry>, Query<'a, &'b Material>),
    Query<'a, (Model2D<'b>, Filter<F>)>,
);

fn create_instance(transform: &Transform2D, z_index: Option<&ZIndex2D>) -> Instance {
    let z = z_index.copied().unwrap_or_default().to_normalized_f32();
    Instance {
        transform: (Mat4::from_scale(transform.size.with_z(0.))
            * Quat::from_z(*transform.rotation).matrix()
            * Mat4::from_position(transform.position.with_z(z)))
        .to_array(),
    }
}

pub(crate) mod opaque;
pub(crate) mod transparent;
