use crate::components::renderer::Renderer;
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::ZIndex2D;
use fxhash::FxHashMap;
use modor::{
    Changed, Custom, CustomQuerySystemParam, Entity, EntityFilter, Filter, Or, Query, QueryFilter,
    SingleRef, World,
};
use modor_math::{Mat4, Quat};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource, ResourceRegistry, ResourceState};
use wgpu::{vertex_attr_array, VertexAttribute, VertexStepMode};

pub(crate) type InstanceGroup2DRegistry = ResourceRegistry<InstanceGroup2D>;
type UpdatedInstanceFilter = Or<(Changed<Transform2D>, Changed<ZIndex2D>)>;

/// A group of instances to render.
///
/// As it is possible to use one [`InstanceGroup2D`] per instance to display, it is recommended to
/// put all instances displayed with the same camera, material and mesh in the same
/// [`InstanceGroup2D`] for better rendering performance.
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
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// const RED_RECTANGLE_MATERIAL: ResKey<Material> = ResKey::new("red-rectangle");
/// const GREEN_ELLIPSE_MATERIAL: ResKey<Material> = ResKey::new("green-ellipse");
/// const CAMERA: ResKey<Camera2D> = ResKey::new("main");
///
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .child_component(Material::new(RED_RECTANGLE_MATERIAL))
///         .with(|m| m.color = Color::RED)
///         .child_component(Material::ellipse(GREEN_ELLIPSE_MATERIAL))
///         .with(|m| m.color = Color::GREEN)
///         .child_entity(red_rectangle_instance_group())
///         .child_entity(red_rectangle(Vec2::ZERO, Vec2::new(0.5, 0.2)))
///         .child_entity(red_rectangle(Vec2::new(-0.1, 0.2), Vec2::ONE * 0.1))
///         .child_entity(green_ellipse(Vec2::new(-0.25, 0.25), Vec2::new(0.1, 0.1)))
/// }
///
/// fn red_rectangle_instance_group() -> impl BuiltEntity {
///     let group_key = ResKey::new("green-ellipse");
///     let filter = QueryFilter::new::<With<RedRectangle>>();
///     EntityBuilder::new()
///         .component(InstanceGroup2D::from_filter(group_key, filter))
///         .component(InstanceRendering2D::new(
///             group_key,
///             CAMERA,
///             RED_RECTANGLE_MATERIAL,
///         ))
/// }
///
/// fn red_rectangle(position: Vec2, size: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .with(|t| t.position = position)
///         .with(|t| t.size = size)
///         .component(RedRectangle)
/// }
///
/// fn green_ellipse(position: Vec2, size: Vec2) -> impl BuiltEntity {
///     let group_key = ResKey::new("green-ellipse");
///     EntityBuilder::new()
///         .component(InstanceGroup2D::from_self(group_key))
///         .component(InstanceRendering2D::new(
///             group_key,
///             CAMERA,
///             GREEN_ELLIPSE_MATERIAL,
///         ))
///         .component(Transform2D::new())
///         .with(|t| t.position = position)
///         .with(|t| t.size = size)
/// }
///
/// #[derive(Component, NoSystem)]
/// struct RedRectangle;
/// ```
#[derive(Component, Debug)]
pub struct InstanceGroup2D {
    instances: Option<DynamicBuffer<Instance>>,
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
            instances: None,
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
            instances: None,
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
            self.instances = None;
            self.entity_ids.clear();
            self.entity_positions.clear();
            self.is_initialized = false;
        }
        if let Some(context) = state.context() {
            if !self.is_initialized {
                self.instances = Some(DynamicBuffer::new(
                    vec![],
                    DynamicBufferUsage::Instance,
                    format!("modor_instance_buffer_{}", self.key.label()),
                    &context.device,
                ));
                self.register_instances(entity, instances);
                self.instances_mut().sync(context);
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
                self.register_instances(entity, instances);
                self.instances_mut().sync(context);
            }
        }
    }

    pub(crate) fn buffer(&self) -> &DynamicBuffer<Instance> {
        self.instances
            .as_ref()
            .expect("internal error: instance buffer not initialized")
    }

    fn register_instances<F>(
        &mut self,
        entity: Entity<'_>,
        mut instances: Query<'_, Custom<InstanceEntity<'_, F>>>,
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
    }

    fn register_instance(&mut self, entity_id: usize, instances: Instance) {
        if let Some(&position) = self.entity_positions.get(&entity_id) {
            self.instances_mut()[position] = instances;
            trace!(
                "instance with ID {entity_id} updated in instance group {}", // no-coverage
                self.key.label()                                             // no-coverage
            );
        } else {
            let position = self.entity_ids.len();
            self.entity_positions.insert(entity_id, position);
            self.entity_ids.push(entity_id);
            self.instances_mut().push(instances);
            trace!(
                "instance with ID {entity_id} added in instance group {}", // no-coverage
                self.key.label()                                           // no-coverage
            );
        }
    }

    fn delete_instance(&mut self, entity_id: usize) {
        if let Some(position) = self.entity_positions.remove(&entity_id) {
            self.instances_mut().swap_remove(position);
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

    fn instances_mut(&mut self) -> &mut DynamicBuffer<Instance> {
        self.instances
            .as_mut()
            .expect("internal error: instance buffer not loaded")
    }
}

impl Resource for InstanceGroup2D {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.instances.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::NotLoaded
        }
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
