use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::{ChangedModel2D, GroupKey, Instance, Model2D};
use crate::components::material::MaterialRegistry;
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{Material, Model, Renderer, ZIndex2D};
use fxhash::FxHashMap;
use modor::{Filter, Query, Single, SingleMut, World};
use modor_physics::Transform2D;
use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use std::ops::Range;

#[derive(SingletonComponent, Debug, Default)]
pub(crate) struct TransparentInstanceRegistry {
    buffer: Option<DynamicBuffer<Instance>>,
    instances: Vec<InstanceDetails>,
    entity_positions: EntityPositions,
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
                for position in self.entity_positions.delete_entity(entity_id) {
                    buffer.swap_remove(position);
                    self.instances.swap_remove(position);
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
    fn update_models_2d(
        &mut self,
        renderer: Single<'_, Renderer>,
        (mut material_registry, materials): (SingleMut<'_, MaterialRegistry>, Query<'_, &Material>),
        models_2d: Query<'_, (Model2D<'_>, Filter<ChangedModel2D>)>,
    ) {
        let context = renderer
            .state(&mut None)
            .context()
            .expect("internal error: not initialized GPU context");
        let buffer = self
            .buffer
            .as_mut()
            .expect("internal error: transparent instance buffer not initialized");
        for ((transform, model, z_index, entity), _) in models_2d.iter() {
            let entity_id = entity.id();
            let is_transparent = material_registry
                .get(&model.material_key, &materials)
                .map_or(false, Material::is_transparent);
            for position in self.entity_positions.get(entity_id) {
                self.instances[position].is_updated = false;
            }
            if is_transparent {
                for camera_key in &model.camera_keys {
                    let group_key = GroupKey {
                        camera_key: camera_key.clone(),
                        material_key: model.material_key.clone(),
                        mesh_key: model.mesh_key.clone(),
                    };
                    if let Some(position) = self.entity_positions.add(entity_id, &group_key) {
                        buffer[position] = super::create_instance(transform, z_index);
                        self.instances[position] = InstanceDetails {
                            group_key,
                            position,
                            is_updated: true,
                        };
                    } else {
                        buffer.push(super::create_instance(transform, z_index));
                        self.instances.push(InstanceDetails {
                            group_key,
                            position: buffer.len() - 1,
                            is_updated: true,
                        });
                    };
                }
                debug!("transparent instance with ID {entity_id} registered (new/changed)");
            }
            for position in self.entity_positions.get(entity_id).collect::<Vec<_>>() {
                let instance = &self.instances[position];
                if !instance.is_updated {
                    self.entity_positions
                        .delete_instance(position, &instance.group_key);
                    buffer.swap_remove(position);
                    self.instances.swap_remove(position);
                }
            }
        }
        self.instances.sort_unstable_by(|a, b| {
            Self::compare_instances(&buffer[a.position], &buffer[b.position])
        });
        for (position, instance) in self.instances.iter_mut().enumerate() {
            instance.position = position;
        }
        buffer.sort_unstable_by(Self::compare_instances);
        buffer.sync(context);
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
        let buffer = self
            .buffer
            .as_mut()
            .expect("internal error: transparent instance buffer not initialized");
        buffer.push(instance);
        self.entity_positions.add(entity_id, &group_key);
        self.instances.push(InstanceDetails {
            group_key,
            position: buffer.len() - 1,
            is_updated: true,
        });
    }

    fn compare_instances(a: &Instance, b: &Instance) -> Ordering {
        a.transform[3][2]
            .partial_cmp(&b.transform[3][2])
            .unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug)]
struct InstanceDetails {
    group_key: GroupKey,
    position: usize,
    is_updated: bool,
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
        if let Some(instance) = self.registry.instances.get(self.next_position) {
            let group = &instance.group_key;
            let new_next_position = self.registry.instances[self.next_position..]
                .iter()
                .filter(|i| &i.group_key != group)
                .map(|i| i.position)
                .next()
                .unwrap_or(self.registry.instances.len());
            let item = (
                group,
                self.registry
                    .buffer
                    .as_ref()
                    .expect("internal error: transparent instance buffer not initialized"),
                self.next_position..new_next_position,
            );
            self.next_position = new_next_position;
            Some(item)
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
struct EntityPositions {
    entity_ids: Vec<usize>,
    entity_positions: FxHashMap<usize, FxHashMap<GroupKey, usize>>,
}

impl EntityPositions {
    fn get(&self, entity_id: usize) -> impl Iterator<Item = usize> + '_ {
        self.entity_positions
            .get(&entity_id)
            .into_iter()
            .flat_map(HashMap::values)
            .copied()
    }

    // returns the position if the entity already exists
    fn add(&mut self, entity_id: usize, group_key: &GroupKey) -> Option<usize> {
        if let Some(&position) = self
            .entity_positions
            .get(&entity_id)
            .and_then(|p| p.get(group_key))
        {
            Some(position)
        } else {
            self.entity_positions
                .entry(entity_id)
                .or_insert_with(HashMap::default)
                .entry(group_key.clone())
                .or_insert_with(|| self.entity_ids.len());
            self.entity_ids.push(entity_id);
            None
        }
    }

    fn delete_instance(&mut self, position: usize, group_key: &GroupKey) {
        let entity_id = self.delete_instance_and_fix_moved_instance(position);
        self.entity_positions
            .get_mut(&entity_id)
            .expect("internal error: instance entity not found")
            .remove(group_key)
            .expect("internal error: group key does not correspond to instance position");
    }

    // returns the entity positions before deletion
    fn delete_entity(&mut self, entity_id: usize) -> Vec<usize> {
        let mut positions = self
            .entity_positions
            .remove(&entity_id)
            .map_or_else(Vec::new, |p| p.into_values().collect::<Vec<_>>());
        positions.sort_unstable_by_key(|&i| Reverse(i));
        for &position in &positions {
            self.delete_instance_and_fix_moved_instance(position);
        }
        positions
    }

    // Returns entity ID of the deleted instance
    fn delete_instance_and_fix_moved_instance(&mut self, position: usize) -> usize {
        let entity_idx = self.entity_ids.swap_remove(position);
        if let Some(moved_entity_positions) = self
            .entity_ids
            .get(position)
            .and_then(|i| self.entity_positions.get_mut(i))
        {
            for moved_position in moved_entity_positions.values_mut() {
                if moved_position == &self.entity_ids.len() {
                    *moved_position = position;
                }
            }
        }
        entity_idx
    }
}

#[cfg(test)]
mod entity_positions_tests {
    use crate::components::instances::transparent::EntityPositions;
    use crate::components::instances::GroupKey;
    use modor_resources::IntoResourceKey;

    #[test]
    fn add_new_entities() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        assert_eq!(positions.add(10, &group1), None);
        assert_eq!(positions.add(10, &group2), None);
        assert_eq!(positions.add(20, &group1), None);
        assert_eq!(positions.entity_ids, [10, 10, 20]);
        assert_eq!(positions.entity_positions[&10][&group1], 0);
        assert_eq!(positions.entity_positions[&10][&group2], 1);
        assert_eq!(positions.entity_positions[&20][&group1], 2);
    }

    #[test]
    fn add_existing_entity() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        positions.add(10, &group1);
        positions.add(10, &group2);
        assert_eq!(positions.add(10, &group2), Some(1));
        assert_eq!(positions.entity_ids, [10, 10]);
        assert_eq!(positions.entity_positions[&10][&group1], 0);
        assert_eq!(positions.entity_positions[&10][&group2], 1);
    }

    #[test]
    fn delete_first_instances() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        let group3 = GroupKey {
            camera_key: 7.into_key(),
            material_key: 8.into_key(),
            mesh_key: 9.into_key(),
        };
        positions.add(10, &group1);
        positions.add(10, &group2);
        positions.add(10, &group3);
        positions.add(20, &group1);
        positions.add(30, &group2);
        positions.delete_instance(1, &group2);
        positions.delete_instance(0, &group1);
        assert_eq!(positions.entity_ids, [20, 30, 10]);
        assert_eq!(positions.entity_positions[&10].get(&group1), None);
        assert_eq!(positions.entity_positions[&10].get(&group2), None);
        assert_eq!(positions.entity_positions[&20][&group1], 0);
        assert_eq!(positions.entity_positions[&30][&group2], 1);
        assert_eq!(positions.entity_positions[&10][&group3], 2);
    }

    #[test]
    fn delete_last_instances() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        positions.add(10, &group1);
        positions.add(20, &group2);
        positions.add(30, &group1);
        positions.add(30, &group2);
        positions.delete_instance(3, &group2);
        positions.delete_instance(2, &group1);
        assert_eq!(positions.entity_ids, [10, 20]);
        assert_eq!(positions.entity_positions[&10][&group1], 0);
        assert_eq!(positions.entity_positions[&20][&group2], 1);
        assert_eq!(positions.entity_positions[&30].len(), 0);
    }

    #[test]
    fn delete_first_entities() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        let group3 = GroupKey {
            camera_key: 7.into_key(),
            material_key: 8.into_key(),
            mesh_key: 9.into_key(),
        };
        positions.add(10, &group1);
        positions.add(10, &group2);
        positions.add(10, &group3);
        positions.add(20, &group1);
        positions.add(30, &group2);
        assert_eq!(positions.delete_entity(10), [2, 1, 0]);
        assert_eq!(positions.delete_entity(10), [] as [usize; 0]);
        assert_eq!(positions.entity_ids, [30, 20]);
        assert_eq!(positions.entity_positions.get(&10), None);
        assert_eq!(positions.entity_positions[&30][&group2], 0);
        assert_eq!(positions.entity_positions[&20][&group1], 1);
    }

    #[test]
    #[should_panic = "internal error: group key does not correspond to instance position"]
    fn delete_instance_with_invalid_group_key() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        positions.add(10, &group1);
        positions.delete_instance(0, &group2);
    }

    #[test]
    fn delete_last_entities() {
        let mut positions = EntityPositions::default();
        let group1 = GroupKey {
            camera_key: 1.into_key(),
            material_key: 2.into_key(),
            mesh_key: 3.into_key(),
        };
        let group2 = GroupKey {
            camera_key: 4.into_key(),
            material_key: 5.into_key(),
            mesh_key: 6.into_key(),
        };
        positions.add(10, &group1);
        positions.add(20, &group2);
        positions.add(30, &group1);
        positions.add(30, &group2);
        assert_eq!(positions.delete_entity(30), [3, 2]);
        assert_eq!(positions.delete_entity(30), [] as [usize; 0]);
        assert_eq!(positions.entity_ids, [10, 20]);
        assert_eq!(positions.entity_positions[&10][&group1], 0);
        assert_eq!(positions.entity_positions[&20][&group2], 1);
        assert_eq!(positions.entity_positions.get(&30), None);
    }
}
