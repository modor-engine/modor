use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::instances::opaque::OpaqueInstanceRegistry;
use crate::instances::{ChangedModel2D, GroupKey, Instance, Model2D};
use crate::resources::material::MaterialRegistry;
use crate::{GraphicsModule, Material, Model};
use fxhash::FxHashMap;
use modor::{Filter, Query, Single, SingleMut, World};
use modor_physics::Transform2D;
use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use std::ops::Range;
use wgpu::Device;

#[derive(SingletonComponent, Debug)]
pub(crate) struct TransparentInstanceRegistry {
    buffer: DynamicBuffer<Instance>,
    instances: Vec<InstanceDetails>,
    entity_positions: EntityPositions,
}

#[systems]
impl TransparentInstanceRegistry {
    pub(crate) fn new(device: &Device) -> Self {
        Self {
            buffer: DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                "transparent_instances",
                device,
            ),
            instances: vec![],
            entity_positions: EntityPositions::default(),
        }
    }

    #[run_after(component(OpaqueInstanceRegistry))]
    fn delete_models(&mut self, world: World<'_>) {
        let deleted_entity_ids = world
            .transformed_entity_ids()
            .chain(world.deleted_entity_ids());
        for entity_id in deleted_entity_ids {
            for position in self.entity_positions.delete(entity_id) {
                self.buffer.swap_remove(position);
                self.instances.swap_remove(position);
            }
        }
    }

    #[run_after_previous_and(component(Transform2D), component(Model))]
    fn update_models_2d(
        &mut self,
        models_2d: Query<'_, (Model2D<'_>, Filter<ChangedModel2D>)>,
        module: Single<'_, GraphicsModule>,
        (mut material_registry, materials): (SingleMut<'_, MaterialRegistry>, Query<'_, &Material>),
    ) {
        for ((transform, model, z_index, entity), _) in models_2d.iter() {
            let is_transparent = material_registry
                .get(&model.material_key, &materials)
                .map_or(false, Material::is_transparent);
            if !is_transparent {
                continue;
            }
            for camera_key in &model.camera_keys {
                let group_key = GroupKey {
                    camera_key: camera_key.clone(),
                    material_key: model.material_key.clone(),
                    mesh_key: model.mesh_key.clone(),
                };
                if let Some(position) = self.entity_positions.add(entity.id(), &group_key) {
                    self.buffer[position] = super::create_instance(transform, z_index, 0.);
                    self.instances[position] = InstanceDetails {
                        group: group_key,
                        position,
                    };
                } else {
                    self.buffer
                        .push(super::create_instance(transform, z_index, 0.));
                    self.instances.push(InstanceDetails {
                        group: group_key,
                        position: self.buffer.len() - 1,
                    });
                };
            }
        }
        self.instances.sort_unstable_by(|a, b| {
            Self::compare_instances(&self.buffer[a.position], &self.buffer[b.position])
        });
        for (position, instance) in self.instances.iter_mut().enumerate() {
            instance.position = position;
        }
        self.buffer.sort_unstable_by(Self::compare_instances);
        self.buffer.sync(&module);
    }

    pub(crate) fn iter(&self) -> GroupIterator<'_> {
        GroupIterator::new(self)
    }

    pub(super) fn add_opaque_instance(
        &mut self,
        instance: Instance,
        entity_id: usize,
        group_key: GroupKey,
    ) {
        self.buffer.push(instance);
        self.entity_positions.add(entity_id, &group_key);
        self.instances.push(InstanceDetails {
            group: group_key,
            position: self.buffer.len() - 1,
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
    group: GroupKey,
    position: usize,
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
            let group = &instance.group;
            let new_next_position = self.registry.instances[self.next_position..]
                .iter()
                .filter(|i| &i.group != group)
                .map(|i| i.position)
                .next()
                .unwrap_or(self.registry.instances.len());
            let item = (
                group,
                &self.registry.buffer,
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

    // returns the entity positions before deletion
    fn delete(&mut self, entity_id: usize) -> Vec<usize> {
        let mut positions = self
            .entity_positions
            .remove(&entity_id)
            .expect("internal error: entity not found in opaque instance")
            .into_values()
            .collect::<Vec<_>>();
        positions.sort_unstable_by_key(|&i| Reverse(i));
        for &position in &positions {
            self.entity_ids.swap_remove(position);
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
        }
        positions
    }
}

#[cfg(test)]
mod entity_positions_tests {
    use crate::instances::transparent::EntityPositions;
    use crate::instances::GroupKey;
    use crate::IntoResourceKey;

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
    fn remove_first_entities() {
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
        assert_eq!(positions.delete(10), [2, 1, 0]);
        assert_eq!(positions.entity_ids, [30, 20]);
        assert_eq!(positions.entity_positions.get(&10), None);
        assert_eq!(positions.entity_positions[&30][&group2], 0);
        assert_eq!(positions.entity_positions[&20][&group1], 1);
    }

    #[test]
    fn remove_last_entities() {
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
        assert_eq!(positions.delete(30), [3, 2]);
        assert_eq!(positions.entity_ids, [10, 20]);
        assert_eq!(positions.entity_positions[&10][&group1], 0);
        assert_eq!(positions.entity_positions[&20][&group2], 1);
        assert_eq!(positions.entity_positions.get(&30), None);
    }
}
