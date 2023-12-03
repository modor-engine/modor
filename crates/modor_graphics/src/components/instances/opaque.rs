use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::instances::{
    ChangedModel2DFilter, Graphics2DResources, GraphicsEntity2D, GroupKey, GroupKeyState, Instance,
};
use crate::components::material::MaterialRegistry;
use crate::components::mesh::Mesh;
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{Camera2D, GpuContext, Material, Model, Renderer, ZIndex2D};
use fxhash::FxHashMap;
use modor::{Custom, CustomQuerySystemParam, EntityFilter, Query, SingleMut, SingleRef, World};
use modor_physics::Transform2D;
use modor_resources::{Resource, ResourceRegistry};
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
        mut transparent_instances: SingleMut<'_, '_, TransparentInstanceRegistry>,
    ) {
        let transparent_instances = transparent_instances.get_mut();
        for material in materials.iter().filter(|m| m.is_newly_transparent()) {
            let moved_group_keys = self
                .groups
                .keys()
                .filter(|k| k.material_key == material.key())
                .copied()
                .collect::<Vec<_>>();
            for group_key in moved_group_keys {
                let group = self
                    .groups
                    .remove(&group_key)
                    .expect("internal error: opaque group not found");
                for (entity_id, instance) in group.into_iter() {
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
    fn update_models_2d(
        &mut self,
        resources: Custom<Graphics2DResources<'_, ChangedModel2DFilter>>,
    ) {
        if self.is_initialized {
            self.register_models_2d(resources);
        }
    }

    #[run_after_previous]
    fn init_models_2d(&mut self, resources: Custom<Graphics2DResources<'_, ()>>) {
        if !self.is_initialized {
            self.register_models_2d(resources);
            self.is_initialized = true;
        }
    }

    #[run_after_previous]
    fn delete_groups(
        &mut self,
        cameras: SingleRef<'_, '_, ResourceRegistry<Camera2D>>,
        materials: SingleRef<'_, '_, ResourceRegistry<Material>>,
        meshes: SingleRef<'_, '_, ResourceRegistry<Mesh>>,
    ) {
        let cameras = cameras.get();
        let materials = materials.get();
        let meshes = meshes.get();
        self.groups.retain(|k, g| {
            g.buffer.len() > 0
                || (cameras.exists(k.camera_key)
                    && materials.exists(k.material_key)
                    && meshes.exists(k.mesh_key))
        });
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (GroupKey, &DynamicBuffer<Instance>)> {
        self.groups
            .iter()
            .map(|(&k, g)| (k, &g.buffer))
            .filter(|(_, g)| g.len() > 0)
    }

    fn register_models_2d<F>(&mut self, resources: Custom<Graphics2DResources<'_, F>>)
    where
        F: EntityFilter,
    {
        let context = resources
            .renderer
            .get()
            .state(&mut None)
            .context()
            .expect("internal error: not initialized GPU context");
        for (entity, _) in resources.models.iter() {
            let entity_id = entity.entity.id();
            let is_transparent = resources
                .materials
                .get(entity.model.material_key)
                .map_or(false, Material::is_transparent);
            self.reset_entity_state(entity_id);
            if !is_transparent {
                for &camera_key in &entity.model.camera_keys {
                    let group_key = GroupKey {
                        camera_key,
                        material_key: entity.model.material_key,
                        mesh_key: entity.model.mesh_key,
                    };
                    let is_new = self.instances_or_create(context, group_key).add(&entity);
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
                    self.delete_entity_from_group(entity_id, s.group_key);
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
        group_key: GroupKey,
    ) -> &mut InstanceGroup {
        self.groups
            .entry(group_key)
            .or_insert_with(|| InstanceGroup::new(context, group_key))
    }

    fn register_entity_in_group(&mut self, entity_id: usize, group_key: GroupKey) {
        self.entity_states
            .entry(entity_id)
            .or_default()
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
            self.delete_entity_from_group(entity_id, state.group_key);
        }
    }

    fn delete_entity_from_group(&mut self, entity_id: usize, group_key: GroupKey) {
        if let Some(group) = self.groups.get_mut(&group_key) {
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
    fn new(context: &GpuContext, key: GroupKey) -> Self {
        Self {
            buffer: DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                format!("modor_instance_buffer_opaque_{key:?}"),
                &context.device,
            ),
            entity_ids: vec![],
            entity_positions: FxHashMap::default(),
        }
    }

    // returns if the model is new
    fn add(
        &mut self,
        entity: &<GraphicsEntity2D<'_> as CustomQuerySystemParam>::ConstParam<'_>,
    ) -> bool {
        if let Some(&position) = self.entity_positions.get(&entity.entity.id()) {
            self.buffer[position] = super::create_instance(entity);
            false
        } else {
            let position = self.entity_ids.len();
            self.entity_positions.insert(entity.entity.id(), position);
            self.entity_ids.push(entity.entity.id());
            self.buffer.push(super::create_instance(entity));
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
