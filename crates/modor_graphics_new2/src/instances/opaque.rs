use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::instances::transparent::TransparentInstanceRegistry;
use crate::instances::{ChangedModel2D, GroupKey, Instance, Model2D};
use crate::resources::material::MaterialRegistry;
use crate::{GraphicsModule, Material, Model, Resource};
use fxhash::FxHashMap;
use modor::{Filter, Query, Single, SingleMut, World};
use modor_physics::Transform2D;

#[derive(SingletonComponent, Debug, Default)]
pub(crate) struct OpaqueInstanceRegistry {
    groups: FxHashMap<GroupKey, InstanceGroup>,
    entity_groups: FxHashMap<usize, Vec<GroupKey>>,
}

#[systems]
impl OpaqueInstanceRegistry {
    #[run]
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
                let entity_ids = group.entity_positions.entity_ids;
                for (instance, entity_id) in group.buffer.iter().zip(entity_ids) {
                    let group_key = group_key.clone();
                    transparent_instances.add_opaque_instance(*instance, entity_id, group_key);
                    self.delete_entity(entity_id);
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
        }
    }

    #[run_after_previous_and(
        component(Transform2D),
        component(Model),
        component(MaterialRegistry),
        component(Material)
    )]
    fn update_models_2d(
        &mut self,
        module: Single<'_, GraphicsModule>,
        models_2d: Query<'_, (Model2D<'_>, Filter<ChangedModel2D>)>,
        (mut material_registry, materials): (SingleMut<'_, MaterialRegistry>, Query<'_, &Material>),
    ) {
        for ((transform, model, z_index, entity), _) in models_2d.iter() {
            let is_transparent = material_registry
                .get(&model.material_key, &materials)
                .map_or(false, Material::is_transparent);
            if is_transparent {
                continue;
            }
            for camera_key in &model.camera_keys {
                let group_key = GroupKey {
                    camera_key: camera_key.clone(),
                    material_key: model.material_key.clone(),
                    mesh_key: model.mesh_key.clone(),
                };
                let is_new = self
                    .groups
                    .entry(group_key.clone())
                    .or_insert_with(|| InstanceGroup::new(&module, &group_key))
                    .add((transform, model, z_index, entity));
                if is_new {
                    self.entity_groups
                        .entry(entity.id())
                        .or_insert_with(Vec::new)
                        .push(group_key);
                }
            }
        }
        for group in self.groups.values_mut() {
            group.sync(&module);
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&GroupKey, &DynamicBuffer<Instance>)> {
        self.groups.iter().map(|(k, g)| (k, &g.buffer))
    }

    fn delete_entity(&mut self, entity_id: usize) {
        for group_key in self.entity_groups.remove(&entity_id).iter().flatten() {
            if let Some(group) = self.groups.get_mut(group_key) {
                group.delete(entity_id);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct InstanceGroup {
    buffer: DynamicBuffer<Instance>,
    entity_positions: EntityPositions,
}

impl InstanceGroup {
    fn new(module: &GraphicsModule, key: &GroupKey) -> Self {
        Self {
            buffer: DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                &format!("opaque_instances_{key:?}"),
                &module.device,
            ),
            entity_positions: EntityPositions::default(),
        }
    }

    // returns if the model is new
    fn add(&mut self, (transform, _model, z_index, entity): Model2D<'_>) -> bool {
        if let Some(position) = self.entity_positions.add(entity.id()) {
            self.buffer[position] = super::create_instance(transform, z_index, 0.);
            false
        } else {
            self.buffer
                .push(super::create_instance(transform, z_index, 0.));
            true
        }
    }

    fn delete(&mut self, entity_id: usize) {
        let position = self.entity_positions.delete(entity_id);
        self.buffer.swap_remove(position);
    }

    fn sync(&mut self, module: &GraphicsModule) {
        self.buffer.sync(module);
    }
}

#[derive(Debug, Default)]
struct EntityPositions {
    entity_ids: Vec<usize>,
    entity_positions: FxHashMap<usize, usize>,
}

impl EntityPositions {
    // returns the position if the entity already exists
    fn add(&mut self, entity_id: usize) -> Option<usize> {
        if let Some(&position) = self.entity_positions.get(&entity_id) {
            Some(position)
        } else {
            self.entity_positions
                .insert(entity_id, self.entity_ids.len());
            self.entity_ids.push(entity_id);
            None
        }
    }

    // returns the entity position before deletion
    fn delete(&mut self, entity_id: usize) -> usize {
        let position = self
            .entity_positions
            .remove(&entity_id)
            .expect("internal error: entity not found in opaque instance");
        self.entity_ids.swap_remove(position);
        if let Some(moved_entity_id) = self.entity_ids.get(position) {
            let last_entity_position = self
                .entity_positions
                .get_mut(moved_entity_id)
                .expect("internal error: last entity position not found in opaque instance");
            *last_entity_position = position;
        }
        position
    }
}

#[cfg(test)]
mod entity_positions_tests {
    use crate::instances::opaque::EntityPositions;

    #[test]
    fn add_new_entities() {
        let mut positions = EntityPositions::default();
        assert_eq!(positions.add(10), None);
        assert_eq!(positions.add(20), None);
        assert_eq!(positions.entity_ids, [10, 20]);
        assert_eq!(positions.entity_positions[&10], 0);
        assert_eq!(positions.entity_positions[&20], 1);
    }

    #[test]
    fn add_existing_entity() {
        let mut positions = EntityPositions::default();
        positions.add(10);
        assert_eq!(positions.add(10), Some(0));
        assert_eq!(positions.entity_ids, [10]);
        assert_eq!(positions.entity_positions[&10], 0);
    }

    #[test]
    fn remove_first_entity() {
        let mut positions = EntityPositions::default();
        positions.add(10);
        positions.add(20);
        positions.add(30);
        assert_eq!(positions.delete(10), 0);
        assert_eq!(positions.entity_ids, [30, 20]);
        assert_eq!(positions.entity_positions.get(&10), None);
        assert_eq!(positions.entity_positions[&20], 1);
        assert_eq!(positions.entity_positions[&30], 0);
    }

    #[test]
    fn remove_last_entity() {
        let mut positions = EntityPositions::default();
        positions.add(10);
        positions.add(20);
        positions.add(30);
        assert_eq!(positions.delete(30), 2);
        assert_eq!(positions.entity_ids, [10, 20]);
        assert_eq!(positions.entity_positions[&10], 0);
        assert_eq!(positions.entity_positions[&20], 1);
        assert_eq!(positions.entity_positions.get(&30), None);
    }
}
