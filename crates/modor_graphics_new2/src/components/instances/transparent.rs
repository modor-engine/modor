use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::{ChangedModel2D, GroupKey, Instance, Model2DResources};
use crate::components::material::MaterialRegistry;
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{Material, Model, Renderer, ZIndex2D};
use fxhash::FxHashMap;
use modor::{EntityFilter, Single, World};
use modor_physics::Transform2D;
use std::collections::HashMap;
use std::ops::Range;

#[derive(SingletonComponent, Debug, Default)]
pub(crate) struct TransparentInstanceRegistry {
    buffer: Option<DynamicBuffer<Instance>>,
    instances: InstanceDetails,
    is_initialized: bool,
}

#[systems]
impl TransparentInstanceRegistry {
    #[run_after(component(Renderer))]
    fn init_buffer(&mut self, renderer: Single<'_, Renderer>) {
        let context = renderer
            .state(&mut None)
            .context()
            .expect("internal error: not initialized GPU context");
        if self.buffer.is_none() {
            self.buffer = Some(DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                "transparent_instances",
                &context.device,
            ));
        }
    }

    #[run_after_previous_and(component(OpaqueInstanceRegistry))]
    fn delete_models(&mut self, world: World<'_>) {
        if let Some(buffer) = &mut self.buffer {
            let deleted_entity_ids = world
                .transformed_entity_ids()
                .chain(world.deleted_entity_ids());
            for entity_id in deleted_entity_ids {
                for position in self.instances.delete_entity(entity_id) {
                    buffer.swap_remove(position);
                }
                debug!("transparent instance with ID {entity_id} unregistered (changed/deleted)");
            }
        }
    }

    #[run_after_previous_and(
        component(Renderer),
        component(MaterialRegistry),
        component(Material),
        component(Transform2D),
        component(Model),
        component(ZIndex2D)
    )]
    fn update_models_2d(&mut self, resources: Model2DResources<'_, '_, ChangedModel2D>) {
        if self.is_initialized {
            self.register_models_2d(resources);
        }
    }

    #[run_after_previous]
    fn init_models_2d(&mut self, resources: Model2DResources<'_, '_, ()>) {
        if !self.is_initialized {
            self.register_models_2d(resources);
            self.is_initialized = true;
        }
    }

    pub(crate) fn iter(&self) -> GroupIterator<'_> {
        GroupIterator::new(self)
    }

    pub(crate) fn add_opaque_instance(
        &mut self,
        instance: Instance,
        entity_id: usize,
        group_key: GroupKey,
    ) {
        Self::buffer_mut(&mut self.buffer).push(instance);
        self.instances.add(entity_id, group_key);
    }

    fn register_models_2d<F>(
        &mut self,
        (renderer, (mut material_registry, materials), models_2d): Model2DResources<'_, '_, F>,
    ) where
        F: EntityFilter,
    {
        let context = renderer
            .state(&mut None)
            .context()
            .expect("internal error: not initialized GPU context");
        let buffer = Self::buffer_mut(&mut self.buffer);
        for ((transform, model, z_index, entity), _) in models_2d.iter() {
            let entity_id = entity.id();
            let is_transparent = material_registry
                .get(&model.material_key, &materials)
                .map_or(false, Material::is_transparent);
            self.instances.reset_entity_update_state(entity_id);
            if is_transparent {
                for camera_key in &model.camera_keys {
                    let group_key = GroupKey {
                        camera_key: camera_key.clone(),
                        material_key: model.material_key.clone(),
                        mesh_key: model.mesh_key.clone(),
                    };
                    if let Some(position) = self.instances.add(entity_id, group_key) {
                        buffer[position] = super::create_instance(transform, z_index);
                    } else {
                        buffer.push(super::create_instance(transform, z_index));
                    }
                }
                debug!("transparent instance with ID {entity_id} registered (new/changed)");
            }
            for position in self.instances.delete_not_updated(entity_id) {
                buffer.swap_remove(position);
            }
        }
        self.instances.sort(buffer);
        buffer.sort_unstable_by(Instance::cmp_z);
        buffer.sync(context);
    }

    fn buffer(buffer: &Option<DynamicBuffer<Instance>>) -> &DynamicBuffer<Instance> {
        buffer
            .as_ref()
            .expect("internal error: transparent instance buffer not initialized")
    }

    fn buffer_mut(buffer: &mut Option<DynamicBuffer<Instance>>) -> &mut DynamicBuffer<Instance> {
        buffer
            .as_mut()
            .expect("internal error: transparent instance buffer not initialized")
    }
}

#[derive(Debug, Default)]
struct InstanceDetails {
    instances: Vec<InstanceProperties>,
    entity_positions: FxHashMap<usize, FxHashMap<GroupKey, usize>>,
}

impl InstanceDetails {
    fn next_group(&self, first_position: usize) -> Option<(&GroupKey, Range<usize>)> {
        self.instances.get(first_position).map(|instance| {
            let group_key = &instance.group_key;
            let next_range_position = self.instances[first_position..]
                .iter()
                .filter(|i| &i.group_key != group_key)
                .map(|i| i.position)
                .next()
                .unwrap_or(self.instances.len());
            (group_key, first_position..next_range_position)
        })
    }

    // returns existing position if the instance already exists, else returns None
    fn add(&mut self, entity_id: usize, group_key: GroupKey) -> Option<usize> {
        let entity_positions = self
            .entity_positions
            .entry(entity_id)
            .or_insert_with(FxHashMap::default);
        if let Some(&position) = entity_positions.get(&group_key) {
            self.instances[position].is_updated = true;
            Some(position)
        } else {
            let position = self.instances.len();
            entity_positions.insert(group_key.clone(), position);
            self.instances.push(InstanceProperties {
                entity_id,
                group_key,
                position,
                is_updated: true,
            });
            None
        }
    }

    fn delete(&mut self, position: usize) {
        let instance = self.instances.swap_remove(position);
        if let Some(positions) = self.entity_positions.get_mut(&instance.entity_id) {
            positions.remove(&instance.group_key);
        }
        if let Some(instance) = self.instances.get_mut(position) {
            instance.position = position;
            Self::set_position(
                instance.entity_id,
                &instance.group_key,
                position,
                &mut self.entity_positions,
            );
        };
    }

    fn sort(&mut self, buffer: &DynamicBuffer<Instance>) {
        self.instances
            .sort_unstable_by(|a, b| buffer[a.position].cmp_z(&buffer[b.position]));
        for (position, instance) in self.instances.iter_mut().enumerate() {
            instance.position = position;
        }
        for instance in &self.instances {
            Self::set_position(
                instance.entity_id,
                &instance.group_key,
                instance.position,
                &mut self.entity_positions,
            );
        }
    }

    // returns the ordered positions of deleted instances
    fn delete_entity(&mut self, entity_id: usize) -> impl Iterator<Item = usize> + '_ {
        self.entity_positions
            .remove(&entity_id)
            .into_iter()
            .flat_map(HashMap::into_values)
            .inspect(|&p| self.delete(p))
    }

    fn reset_entity_update_state(&mut self, entity_id: usize) {
        for &position in self
            .entity_positions
            .get(&entity_id)
            .iter()
            .flat_map(|p| p.values())
        {
            self.instances[position].is_updated = false;
        }
    }

    // returns the ordered positions of deleted instances
    fn delete_not_updated(&mut self, entity_id: usize) -> Vec<usize> {
        let deleted_positions = self
            .entity_positions
            .get(&entity_id)
            .iter()
            .flat_map(|p| p.values())
            .copied()
            .filter(|&p| !self.instances[p].is_updated)
            .collect::<Vec<_>>();
        for &position in &deleted_positions {
            self.delete(position);
        }
        deleted_positions
    }

    fn set_position(
        entity_id: usize,
        group_key: &GroupKey,
        position: usize,
        entity_positions: &mut FxHashMap<usize, FxHashMap<GroupKey, usize>>,
    ) {
        *entity_positions
            .get_mut(&entity_id)
            .expect("internal error: not found position of entity")
            .get_mut(group_key)
            .expect("internal error: not found position of entity group") = position;
    }
}

pub(crate) struct GroupIterator<'a> {
    registry: &'a TransparentInstanceRegistry,
    next_position: usize,
}

impl<'a> GroupIterator<'a> {
    fn new(registry: &'a TransparentInstanceRegistry) -> Self {
        Self {
            registry,
            next_position: 0,
        }
    }
}

impl<'a> Iterator for GroupIterator<'a> {
    type Item = (&'a GroupKey, &'a DynamicBuffer<Instance>, Range<usize>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((group_key, range)) = self.registry.instances.next_group(self.next_position) {
            self.next_position = range.end;
            Some((
                group_key,
                TransparentInstanceRegistry::buffer(&self.registry.buffer),
                range,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct InstanceProperties {
    entity_id: usize,
    group_key: GroupKey,
    position: usize,
    is_updated: bool,
}
