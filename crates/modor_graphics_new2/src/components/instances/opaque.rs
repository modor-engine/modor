use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::instances::{
    ChangedModel2D, GroupKey, GroupKeyState, Instance, Model2D, Model2DResources,
};
use crate::components::material::MaterialRegistry;
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{GpuContext, Material, Model, Renderer, ZIndex2D};
use fxhash::FxHashMap;
use modor::{EntityFilter, Query, SingleMut, World};
use modor_physics::Transform2D;
use modor_resources::Resource;
use std::iter::Zip;
use std::mem;
use std::vec::IntoIter;

#[derive(SingletonComponent, Debug, Default)]
pub(crate) struct OpaqueInstanceRegistry {
    groups: FxHashMap<GroupKey, InstanceGroup>,
    entity_states: FxHashMap<usize, Vec<GroupKeyState>>,
    is_initialized: bool,
}

#[systems]
impl OpaqueInstanceRegistry {
    #[run_after(component(Material))]
    fn move_transparent(
        &mut self,
        materials: Query<'_, &Material>,
        mut transparent_instances: SingleMut<'_, TransparentInstanceRegistry>,
    ) {
        for material in materials.iter().filter(|m| m.is_newly_transparent()) {
            let moved_group_keys = self
                .groups
                .keys()
                .filter(|k| &k.material_key == material.key())
                .cloned()
                .collect::<Vec<_>>();
            for group_key in moved_group_keys {
                let group = self
                    .groups
                    .remove(&group_key)
                    .expect("internal error: opaque group not found");
                for (entity_id, instance) in group.into_iter() {
                    let group_key = group_key.clone();
                    transparent_instances.add_opaque_instance(instance, entity_id, group_key);
                    self.delete_entity(entity_id);
                    debug!("opaque instance with ID `{entity_id}` is now transparent");
                }
            }
        }
    }

    #[run_after_previous]
    fn delete_models(&mut self, world: World<'_>) {
        let deleted_entity_ids = world
            .transformed_entity_ids()
            .chain(world.deleted_entity_ids());
        for entity_id in deleted_entity_ids {
            self.delete_entity(entity_id);
            debug!("opaque instance with ID {entity_id} unregistered (changed/deleted)");
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&GroupKey, &DynamicBuffer<Instance>)> {
        self.groups.iter().map(|(k, g)| (k, &g.buffer))
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
        for ((transform, model, z_index, entity), _) in models_2d.iter() {
            let entity_id = entity.id();
            let is_transparent = material_registry
                .get(&model.material_key, &materials)
                .map_or(false, Material::is_transparent);
            self.reset_entity_state(entity_id);
            if !is_transparent {
                for camera_key in &model.camera_keys {
                    let group_key = GroupKey {
                        camera_key: camera_key.clone(),
                        material_key: model.material_key.clone(),
                        mesh_key: model.mesh_key.clone(),
                    };
                    let is_new = self
                        .instances_or_create(context, &group_key)
                        .add((transform, model, z_index, entity));
                    if is_new {
                        self.register_entity_in_group(entity_id, group_key);
                    } else {
                        self.update_entity_in_group(entity_id, group_key);
                    }
                }
                debug!("opaque instance with ID {entity_id} registered (new/changed)");
            }
            self.remove_entity_from_not_updated_groups(entity_id);
        }
        for group in self.groups.values_mut() {
            group.sync(context);
        }
    }

    fn reset_entity_state(&mut self, entity_id: usize) {
        if let Some(states) = self.entity_states.get_mut(&entity_id) {
            for state in states {
                state.is_updated = false;
            }
        }
    }

    fn remove_entity_from_not_updated_groups(&mut self, entity_id: usize) {
        let new_states = self
            .entity_states
            .get_mut(&entity_id)
            .map_or_else(Vec::new, mem::take)
            .into_iter()
            .filter(|s| {
                if s.is_updated {
                    true
                } else {
                    self.delete_entity_from_group(entity_id, &s.group_key);
                    false
                }
            })
            .collect();
        if let Some(states) = self.entity_states.get_mut(&entity_id) {
            *states = new_states;
        }
    }

    fn instances_or_create(
        &mut self,
        context: &GpuContext,
        group_key: &GroupKey,
    ) -> &mut InstanceGroup {
        self.groups
            .entry(group_key.clone())
            .or_insert_with(|| InstanceGroup::new(context, group_key))
    }

    fn register_entity_in_group(&mut self, entity_id: usize, group_key: GroupKey) {
        self.entity_states
            .entry(entity_id)
            .or_insert_with(Vec::new)
            .push(GroupKeyState {
                group_key,
                is_updated: true,
            });
    }

    fn update_entity_in_group(&mut self, entity_id: usize, group_key: GroupKey) {
        for state in self
            .entity_states
            .get_mut(&entity_id)
            .iter_mut()
            .flat_map(|s| s.iter_mut())
        {
            if state.group_key == group_key {
                state.is_updated = true;
            }
        }
    }

    fn delete_entity(&mut self, entity_id: usize) {
        for state in self.entity_states.remove(&entity_id).iter().flatten() {
            self.delete_entity_from_group(entity_id, &state.group_key);
        }
    }

    fn delete_entity_from_group(&mut self, entity_id: usize, group_key: &GroupKey) {
        if let Some(group) = self.groups.get_mut(group_key) {
            group.delete(entity_id);
        }
    }
}

#[derive(Debug)]
pub(crate) struct InstanceGroup {
    buffer: DynamicBuffer<Instance>,
    entity_ids: Vec<usize>,
    entity_positions: FxHashMap<usize, usize>,
}

impl InstanceGroup {
    fn new(context: &GpuContext, key: &GroupKey) -> Self {
        Self {
            buffer: DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                format!("opaque_instances_{key:?}"),
                &context.device,
            ),
            entity_ids: vec![],
            entity_positions: FxHashMap::default(),
        }
    }

    // returns if the model is new
    fn add(&mut self, (transform, _model, z_index, entity): Model2D<'_>) -> bool {
        if let Some(&position) = self.entity_positions.get(&entity.id()) {
            self.buffer[position] = super::create_instance(transform, z_index);
            false
        } else {
            let position = self.entity_ids.len();
            self.entity_positions.insert(entity.id(), position);
            self.entity_ids.push(entity.id());
            self.buffer.push(super::create_instance(transform, z_index));
            true
        }
    }

    fn delete(&mut self, entity_id: usize) {
        let position = self
            .entity_positions
            .remove(&entity_id)
            .expect("internal error: entity not found in instance group");
        self.buffer.swap_remove(position);
        self.entity_ids.swap_remove(position);
        if let Some(moved_entity_id) = self.entity_ids.get(position) {
            let last_entity_position = self
                .entity_positions
                .get_mut(moved_entity_id)
                .expect("internal error: last entity position not found in opaque instance");
            *last_entity_position = position;
        }
    }

    fn sync(&mut self, context: &GpuContext) {
        self.buffer.sync(context);
    }

    fn into_iter(self) -> Zip<IntoIter<usize>, IntoIter<Instance>> {
        self.entity_ids.into_iter().zip(Vec::from(self.buffer))
    }
}
