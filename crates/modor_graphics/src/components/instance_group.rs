use crate::components::renderer::{GpuContext, Renderer};
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{InstanceData, ZIndex2D};
use bytemuck::Pod;
use fxhash::FxHashMap;
use modor::{
    Changed, Custom, CustomQuerySystemParam, Entity, EntityFilter, Filter, Or, Query, QueryFilter,
    SingleRef, World,
};
use modor_math::{Mat4, Quat};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource, ResourceRegistry, ResourceState};
use std::any::TypeId;
use std::{any, mem};
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

pub(crate) type InstanceGroup2DRegistry = ResourceRegistry<InstanceGroup2D>;
pub(crate) type InstanceDataUpdateQuery<'a, T> = Query<
    'a,
    (
        <T as InstanceData>::Query,
        Entity<'static>,
        Filter<<T as InstanceData>::UpdateFilter>,
    ),
>;
type UpdatedInstanceFilter = Or<(Changed<Transform2D>, Changed<ZIndex2D>)>;

/// A group of instances to render.
///
/// # Requirements
///
/// Instances are rendered only if:
/// - graphics [`module`](crate::module()) is initialized
/// - [`InstanceRendering2D`](crate::InstanceRendering2D) is linked to the group
/// - linked instance entities have [`Transform2D`] component
///
/// # Related components
///
/// - [`Transform2D`](Transform2D)
/// - [`ZIndex2D`](ZIndex2D)
///
/// # Entity functions creating this component
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
///
/// # Performance
///
/// As it is possible to use one [`InstanceGroup2D`] per instance to display, it is recommended to
/// put all instances displayed with the same camera, material and mesh in the same
/// [`InstanceGroup2D`] for better rendering performance.
///
/// # Examples
///
/// See [`instance_group_2d`](crate::instance_group_2d()) and
/// [`instance_2d`](crate::instance_2d()) as most of the time these methods will be used
/// to create an instance group.
#[derive(Component, Debug)]
pub struct InstanceGroup2D {
    pub(crate) z_indexes: Vec<f32>,
    buffers: FxHashMap<TypeId, InstanceBuffer>,
    entity_ids: Vec<usize>,
    entity_positions: FxHashMap<usize, usize>,
    is_initialized: bool,
    renderer_version: Option<u8>,
    key: ResKey<Self>,
    filter: Option<QueryFilter>,
}

#[systems]
impl InstanceGroup2D {
    /// Creates a new instance group with a unique `key` that containing only the current entity
    /// as instance.
    pub fn from_self(key: ResKey<Self>) -> Self {
        Self {
            buffers: FxHashMap::default(),
            z_indexes: vec![],
            entity_ids: vec![],
            entity_positions: FxHashMap::default(),
            is_initialized: false,
            renderer_version: None,
            key,
            filter: None,
        }
    }

    /// Creates a new instance group with a unique `key` containing all entities matching `F` as
    /// instances.
    pub fn from_filter(key: ResKey<Self>, filter: QueryFilter) -> Self {
        Self {
            buffers: FxHashMap::default(),
            z_indexes: vec![],
            entity_ids: vec![],
            entity_positions: FxHashMap::default(),
            is_initialized: false,
            renderer_version: None,
            key,
            filter: Some(filter),
        }
    }

    #[run]
    fn delete(&mut self, world: World<'_>) {
        let deleted_entity_ids = world
            .transformed_entity_ids()
            .chain(world.deleted_entity_ids());
        for entity_id in deleted_entity_ids {
            self.delete_instance(entity_id);
        }
    }

    #[run_after_previous_and(component(Transform2D), component(ZIndex2D), component(Renderer))]
    fn init(
        &mut self,
        entity: Entity<'_>,
        instances: Query<'_, Custom<InstanceEntity<'_, ()>>>,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.buffers.clear();
            self.z_indexes.clear();
            self.entity_ids.clear();
            self.entity_positions.clear();
            self.is_initialized = false;
        }
        if let Some(context) = state.context() {
            if !self.is_initialized {
                self.buffers.insert(
                    TypeId::of::<Instance>(),
                    InstanceBuffer::new::<Instance>(context, self.key),
                );
                self.register_instances(entity, instances, context);
                self.is_initialized = true;
            }
        }
    }

    #[run_after_previous]
    fn update(
        &mut self,
        entity: Entity<'_>,
        instances: Query<'_, Custom<InstanceEntity<'_, UpdatedInstanceFilter>>>,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if let Some(context) = state.context() {
            if self.is_initialized {
                self.register_instances(entity, instances, context);
                for buffer in self.buffers.values_mut() {
                    buffer.is_updated = false;
                }
            }
        }
    }

    pub(crate) fn update_material_instances<T>(
        &mut self,
        query: &mut Query<'_, T::Query>,
        filtered_query: &mut InstanceDataUpdateQuery<'_, T>,
        context: &GpuContext,
    ) where
        T: InstanceData,
    {
        let is_already_registered = self.buffers.contains_key(&TypeId::of::<T>());
        let buffer = self
            .buffers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| InstanceBuffer::new::<T>(context, self.key));
        if buffer.is_updated {
            return;
        }
        for &entity_id in &self.entity_ids
            [(buffer.data.len().div_euclid(buffer.item_size))..self.entity_ids.len()]
        {
            let instance = query.get_mut(entity_id).map(T::data).unwrap_or_default();
            buffer.add(instance);
        }
        if is_already_registered {
            for (item, entity, _) in filtered_query.iter_mut() {
                if let Some(&position) = self.entity_positions.get(&entity.id()) {
                    let instance = T::data(item);
                    buffer.replace(instance, position);
                }
            }
        }
        buffer.is_updated = true;
        buffer.sync(context);
    }

    pub(crate) fn buffer(&self, type_id: TypeId) -> Option<&InstanceBuffer> {
        self.buffers.get(&type_id)
    }

    fn register_instances<F>(
        &mut self,
        entity: Entity<'_>,
        mut instances: Query<'_, Custom<InstanceEntity<'_, F>>>,
        context: &GpuContext,
    ) where
        F: EntityFilter,
    {
        if let Some(filter) = self.filter {
            instances.set_iter_filter(filter);
            for instance in instances.iter() {
                self.register_instance(instance.entity.id(), Instance::from_entity(&instance));
            }
        } else if let Some(instance) = instances.get(entity.id()) {
            self.register_instance(instance.entity.id(), Instance::from_entity(&instance));
        }
        self.buffer_mut::<Instance>().sync(context);
    }

    fn register_instance(&mut self, entity_id: usize, instance: Instance) {
        if let Some(&position) = self.entity_positions.get(&entity_id) {
            self.z_indexes[position] = instance.z();
            self.buffer_mut::<Instance>().replace(instance, position);
            trace!(
                "instance with ID {entity_id} updated in instance group {}", // no-coverage
                self.key.label()                                             // no-coverage
            );
        } else {
            let position = self.entity_ids.len();
            self.entity_positions.insert(entity_id, position);
            self.entity_ids.push(entity_id);
            self.z_indexes.push(instance.z());
            self.buffer_mut::<Instance>().add(instance);
            trace!(
                "instance with ID {entity_id} added in instance group {}", // no-coverage
                self.key.label()                                           // no-coverage
            );
        }
    }

    fn delete_instance(&mut self, entity_id: usize) {
        if let Some(position) = self.entity_positions.remove(&entity_id) {
            for buffer in self.buffers.values_mut() {
                buffer.delete(position);
            }
            self.z_indexes.swap_remove(position);
            self.entity_ids.swap_remove(position);
            if let Some(moved_entity_id) = self.entity_ids.get(position) {
                let last_entity_position = self
                    .entity_positions
                    .get_mut(moved_entity_id)
                    .expect("internal error: last entity position not found in opaque instance");
                *last_entity_position = position;
            }
            trace!(
                "instance with ID {entity_id} removed from instance group  {}", // no-coverage
                self.key.label()                                                // no-coverage
            );
        }
    }

    fn buffer_mut<T>(&mut self) -> &mut InstanceBuffer
    where
        T: Pod,
    {
        self.buffers
            .get_mut(&TypeId::of::<T>())
            .expect("internal error: instance buffer not loaded")
    }
}

impl Resource for InstanceGroup2D {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.buffers.is_empty() {
            ResourceState::NotLoaded
        } else {
            ResourceState::Loaded
        }
    }
}

#[derive(Debug)]
pub(crate) struct InstanceBuffer {
    pub(crate) data: DynamicBuffer<u8>,
    pub(crate) item_size: usize,
    is_updated: bool,
}

impl InstanceBuffer {
    fn new<T>(context: &GpuContext, key: ResKey<InstanceGroup2D>) -> Self
    where
        T: Pod,
    {
        Self {
            data: DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                format!(
                    "modor_instance_buffer_{}_{}",
                    key.label(),
                    any::type_name::<T>()
                ),
                &context.device,
            ),
            item_size: mem::size_of::<T>(),
            is_updated: false,
        }
    }

    fn add<T>(&mut self, item: T)
    where
        T: Pod,
    {
        self.data
            .extend_from_slice(bytemuck::cast_slice::<_, u8>(&[item]));
    }

    fn replace<T>(&mut self, item: T, position: usize)
    where
        T: Pod,
    {
        self.data.splice(
            (position * self.item_size)..((position + 1) * self.item_size),
            bytemuck::cast_slice::<_, u8>(&[item]).iter().copied(),
        );
    }

    fn delete(&mut self, position: usize) {
        for i in (0..self.item_size).rev() {
            self.data.swap_remove(position * self.item_size + i);
        }
    }

    fn sync(&mut self, context: &GpuContext) {
        self.data.sync(context);
    }
}

#[allow(unused)]
#[derive(QuerySystemParam)]
pub(crate) struct InstanceEntity<'a, F>
where
    F: EntityFilter,
{
    entity: Entity<'a>,
    transform: &'a Transform2D,
    z_index: Option<&'a ZIndex2D>,
    _filter: Filter<F>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    transform: [[f32; 4]; 4],
}

impl Instance {
    pub(crate) fn from_entity<F>(
        entity: &<InstanceEntity<'_, F> as CustomQuerySystemParam>::ConstParam<'_>,
    ) -> Self
    where
        F: EntityFilter,
    {
        let z = entity
            .z_index
            .copied()
            .unwrap_or_default()
            .to_normalized_f32();
        Self {
            transform: (Mat4::from_scale(entity.transform.size.with_z(0.))
                * Quat::from_z(entity.transform.rotation).matrix()
                * Mat4::from_position(entity.transform.position.with_z(z)))
            .to_array(),
        }
    }

    pub(crate) fn z(&self) -> f32 {
        self.transform[3][2]
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
