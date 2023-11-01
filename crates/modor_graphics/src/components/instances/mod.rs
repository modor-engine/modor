use crate::components::mesh::Mesh;
use crate::components::renderer::Renderer;
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{Camera2D, Material, Model, ZIndex2D};
use modor::{
    Changed, Custom, CustomQuerySystemParam, Entity, EntityFilter, Filter, Or, Query, SingleRef,
};
use modor_math::{Mat4, Quat};
use modor_physics::Transform2D;
use modor_resources::{ResKey, ResourceAccessor};
use std::cmp::Ordering;
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GroupKey {
    pub(crate) camera_key: ResKey<Camera2D>,
    pub(crate) material_key: ResKey<Material>,
    pub(crate) mesh_key: ResKey<Mesh>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GroupKeyState {
    pub(crate) group_key: GroupKey,
    pub(crate) is_updated: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    transform: [[f32; 4]; 4],
}

impl Instance {
    pub(crate) fn cmp_z(&self, other: &Self) -> Ordering {
        self.transform[3][2]
            .partial_cmp(&other.transform[3][2])
            .unwrap_or(Ordering::Equal)
    }
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

#[allow(unused)]
#[derive(QuerySystemParam)]
struct GraphicsEntity2D<'a> {
    transform: &'a Transform2D,
    model: &'a Model,
    z_index: Option<&'a ZIndex2D>,
    entity: Entity<'a>,
}

#[derive(SystemParam)]
struct Graphics2DResources<'a, F>
where
    F: EntityFilter,
{
    renderer: SingleRef<'a, 'static, Renderer>,
    materials: Custom<ResourceAccessor<'a, Material>>,
    models: Query<'a, (Custom<GraphicsEntity2D<'static>>, Filter<F>)>,
}

type ChangedModel2DFilter = Or<(Changed<Transform2D>, Changed<Model>, Changed<ZIndex2D>)>;

fn create_instance(
    entity: &<GraphicsEntity2D<'_> as CustomQuerySystemParam>::ConstParam<'_>,
) -> Instance {
    let z = entity
        .z_index
        .copied()
        .unwrap_or_default()
        .to_normalized_f32();
    Instance {
        transform: (Mat4::from_scale(entity.transform.size.with_z(0.))
            * Quat::from_z(entity.transform.rotation).matrix()
            * Mat4::from_position(entity.transform.position.with_z(z)))
        .to_array(),
    }
}

pub(crate) mod opaque;
pub(crate) mod transparent;
