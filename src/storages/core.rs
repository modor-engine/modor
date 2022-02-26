use std::any::{Any, TypeId};
use std::mem;
use std::sync::Mutex;

use crate::storages::actions::{ActionDependencies, ActionIdx, ActionStorage};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::entities::{EntityIdx, EntityStorage};
use crate::storages::systems::{SystemProperties, SystemStorage};
use crate::storages::updates::UpdateStorage;
use crate::systems::internal::SystemWrapper;
use crate::SystemData;

#[derive(Default)]
pub struct CoreStorage {
    archetypes: ArchetypeStorage,
    entities: EntityStorage,
    components: ComponentStorage,
    actions: ActionStorage,
    systems: SystemStorage,
    updates: Mutex<UpdateStorage>,
}

impl CoreStorage {
    pub(crate) fn archetypes(&self) -> &ArchetypeStorage {
        &self.archetypes
    }

    pub(crate) fn entities(&self) -> &EntityStorage {
        &self.entities
    }

    pub(crate) fn components(&self) -> &ComponentStorage {
        &self.components
    }

    pub(crate) fn systems(&self) -> &SystemStorage {
        &self.systems
    }

    pub(crate) fn set_thread_count(&mut self, count: u32) {
        self.systems.set_thread_count(count);
    }

    pub(crate) fn register_component_type<C>(&mut self) -> ComponentTypeIdx
    where
        C: Any + Sync + Send,
    {
        self.components.type_idx_or_create::<C>()
    }

    pub(crate) fn add_entity_type<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        self.components.add_entity_type::<C>();
    }

    pub(crate) fn add_component_type<C>(
        &mut self,
        src_archetype_idx: ArchetypeIdx,
    ) -> (ComponentTypeIdx, ArchetypeIdx)
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.components.type_idx_or_create::<C>();
        if let Ok(dst_archetype_idx) = self.archetypes.add_component(src_archetype_idx, type_idx) {
            (type_idx, dst_archetype_idx)
        } else {
            (type_idx, src_archetype_idx)
        }
    }

    pub(crate) fn create_entity(
        &mut self,
        archetype_idx: ArchetypeIdx,
        parent_idx: Option<EntityIdx>,
    ) -> (EntityIdx, EntityLocation) {
        let location = EntityLocation {
            idx: archetype_idx,
            pos: self.archetypes.next_entity_pos(archetype_idx),
        };
        let entity_idx = self.entities.create(location, parent_idx);
        let location = EntityLocation {
            idx: archetype_idx,
            pos: self.archetypes.add_entity(entity_idx, archetype_idx),
        };
        (entity_idx, location)
    }

    pub(crate) fn add_component<C>(
        &mut self,
        component: C,
        type_idx: ComponentTypeIdx,
        location: EntityLocation,
        is_singleton: bool,
    ) where
        C: Any + Sync + Send,
    {
        self.components
            .add(type_idx, location, component, is_singleton);
    }

    pub(crate) fn move_entity(
        &mut self,
        src_location: EntityLocation,
        dst_archetype_idx: ArchetypeIdx,
    ) -> EntityLocation {
        let entity_idx = self.archetypes.entity_idxs(src_location.idx)[src_location.pos];
        let dst_type_idxs = self.archetypes.sorted_type_idxs(dst_archetype_idx);
        for &src_type_idx in self.archetypes.sorted_type_idxs(src_location.idx) {
            if dst_type_idxs.binary_search(&src_type_idx).is_ok() {
                self.components
                    .move_(src_type_idx, src_location, dst_archetype_idx);
            } else {
                self.components.delete(src_type_idx, src_location);
            }
        }
        self.archetypes.delete_entity(src_location);
        let dst_location = EntityLocation {
            idx: dst_archetype_idx,
            pos: self.archetypes.add_entity(entity_idx, dst_archetype_idx),
        };
        self.entities.set_location(entity_idx, dst_location);
        Self::update_moved_entity_location(src_location, &self.archetypes, &mut self.entities);
        dst_location
    }

    pub(crate) fn delete_entity(&mut self, entity_idx: EntityIdx) {
        self.entities.delete(entity_idx, |e, l| {
            for &type_idx in self.archetypes.sorted_type_idxs(l.idx) {
                self.components.delete(type_idx, l);
            }
            self.archetypes.delete_entity(l);
            Self::update_moved_entity_location(l, &self.archetypes, e);
        });
    }

    pub(crate) fn add_system(
        &mut self,
        wrapper: SystemWrapper,
        entity_type: TypeId,
        properties: SystemProperties,
        action_type: Option<TypeId>,
        action_dependencies: ActionDependencies,
    ) -> ActionIdx {
        let entity_type_idx = self
            .components
            .type_idx(entity_type)
            .expect("internal error: missing entity type when adding system");
        let action_idx = self.actions.idx_or_create(action_type, action_dependencies);
        self.actions.add_system(action_idx);
        self.systems
            .add(wrapper, entity_type_idx, properties, action_idx);
        action_idx
    }

    pub(crate) fn update(&mut self) {
        let data = SystemData {
            entities: &self.entities,
            components: &self.components,
            archetypes: &self.archetypes,
            actions: &self.actions,
            updates: &self.updates,
        };
        self.systems.run(data);
        let mut updates = mem::take(
            self.updates
                .get_mut()
                .expect("internal error: cannot access to entity actions"),
        );
        // Each type of update is executed in an order that avoids entity index conflicts
        // and make sure index of deleted entities are not used during one update
        for (entity_idx, add_component_fns, deleted_component_type_idxs) in
            updates.changed_entity_drain()
        {
            if let Some(location) = self.entities.location(entity_idx) {
                let mut dst_archetype_idx = location.idx;
                for type_idx in deleted_component_type_idxs {
                    dst_archetype_idx = self.delete_component_type(type_idx, dst_archetype_idx);
                }
                for add_fns in &add_component_fns {
                    dst_archetype_idx = (add_fns.add_type_fn)(self, dst_archetype_idx);
                }
                let dst_location = self.move_entity(location, dst_archetype_idx);
                for add_fns in add_component_fns {
                    (add_fns.add_fn)(self, dst_location);
                }
            }
        }
        for (create_fn, parent_idx) in updates.created_child_entity_drain() {
            if self.entities.location(parent_idx).is_some() {
                create_fn(self);
            }
        }
        for create_fn in updates.created_root_entity_drain() {
            create_fn(self);
        }
        for entity_idx in updates.deleted_entity_drain() {
            if self.entities.location(entity_idx).is_some() {
                self.delete_entity(entity_idx);
            }
        }
    }

    fn delete_component_type(
        &mut self,
        type_idx: ComponentTypeIdx,
        src_archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        if let Ok(dst_archetype_idx) = self
            .archetypes
            .delete_component(src_archetype_idx, type_idx)
        {
            dst_archetype_idx
        } else {
            src_archetype_idx
        }
    }

    fn update_moved_entity_location(
        location: EntityLocation,
        archetypes: &ArchetypeStorage,
        entities: &mut EntityStorage,
    ) {
        let archetype_entity_idxs = archetypes.entity_idxs(location.idx);
        if let Some(&moved_entity_idx) = archetype_entity_idxs.get(location.pos) {
            entities.set_location(moved_entity_idx, location);
        }
    }
}

#[cfg(test)]
mod core_storage_tests {
    use std::any::{Any, TypeId};
    use typed_index_collections::TiVec;

    use crate::storages::actions::ActionDependencies;
    use crate::storages::archetypes::{ArchetypeFilter, ArchetypeStorage, EntityLocation};
    use crate::storages::core::CoreStorage;
    use crate::storages::entities::EntityIdx;
    use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
    use crate::SystemData;

    impl CoreStorage {
        pub(crate) fn system_data(&self) -> SystemData<'_> {
            SystemData {
                entities: &self.entities,
                components: &self.components,
                archetypes: &self.archetypes,
                actions: &self.actions,
                updates: &self.updates,
            }
        }

        pub(crate) fn create_singleton<C>(&mut self, component: C) -> EntityLocation
        where
            C: Any + Sync + Send,
        {
            let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
            let (type_idx, archetype2_idx) = self.add_component_type::<C>(archetype1_idx);
            let (_, location) = self.create_entity(archetype2_idx, None);
            self.add_component(component, type_idx, location, true);
            location
        }

        pub(crate) fn create_entity_with_1_component<C>(
            &mut self,
            component: C,
            parent_idx: Option<EntityIdx>,
        ) -> EntityLocation
        where
            C: Any + Sync + Send,
        {
            let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
            let (type_idx, archetype2_idx) = self.add_component_type::<C>(archetype1_idx);
            let (_, location) = self.create_entity(archetype2_idx, parent_idx);
            self.add_component(component, type_idx, location, false);
            location
        }

        pub(crate) fn create_entity_with_2_components<C1, C2>(
            &mut self,
            component1: C1,
            component2: C2,
            parent_idx: Option<EntityIdx>,
        ) -> EntityLocation
        where
            C1: Any + Sync + Send,
            C2: Any + Sync + Send,
        {
            let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
            let (type1_idx, archetype2_idx) = self.add_component_type::<C1>(archetype1_idx);
            let (type2_idx, archetype3_idx) = self.add_component_type::<C2>(archetype2_idx);
            let (_, location) = self.create_entity(archetype3_idx, parent_idx);
            self.add_component(component1, type1_idx, location, false);
            self.add_component(component2, type2_idx, location, false);
            location
        }

        pub(crate) fn create_entity_with_3_components<C1, C2, C3>(
            &mut self,
            component1: C1,
            component2: C2,
            component3: C3,
            parent_idx: Option<EntityIdx>,
        ) -> EntityLocation
        where
            C1: Any + Sync + Send,
            C2: Any + Sync + Send,
            C3: Any + Sync + Send,
        {
            let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
            let (type1_idx, archetype2_idx) = self.add_component_type::<C1>(archetype1_idx);
            let (type2_idx, archetype3_idx) = self.add_component_type::<C2>(archetype2_idx);
            let (type3_idx, archetype4_idx) = self.add_component_type::<C3>(archetype3_idx);
            let (_, location) = self.create_entity(archetype4_idx, parent_idx);
            self.add_component(component1, type1_idx, location, false);
            self.add_component(component2, type2_idx, location, false);
            self.add_component(component3, type3_idx, location, false);
            location
        }
    }

    #[test]
    fn set_thread_count() {
        let mut storage = CoreStorage::default();
        storage.set_thread_count(3);
        assert_eq!(storage.systems.thread_count(), 3);
    }

    #[test]
    fn configure_entity() {
        let mut storage = CoreStorage::default();
        let type1_idx = storage.register_component_type::<u32>();
        storage.add_entity_type::<u32>();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type2_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let (type3_idx, archetype3_idx) = storage.add_component_type::<i8>(archetype2_idx);
        let (type4_idx, archetype4_idx) = storage.add_component_type::<i8>(archetype3_idx);
        let (entity1_idx, location1) = storage.create_entity(archetype4_idx, None);
        storage.add_component(10_u32, type2_idx, location1, false);
        storage.add_component(20_i8, type3_idx, location1, false);
        let (entity2_idx, location2) = storage.create_entity(archetype4_idx, Some(entity1_idx));
        storage.add_component(30_u32, type2_idx, location2, false);
        storage.add_component(40_i8, type3_idx, location2, false);
        let location3 = storage.move_entity(location1, archetype2_idx);
        assert_eq!(type1_idx, type2_idx);
        assert_eq!(type3_idx, type4_idx);
        assert_eq!(archetype3_idx, archetype4_idx);
        assert_eq!(storage.entities().location(entity1_idx), Some(location3));
        assert_eq!(storage.entities().location(entity2_idx), Some(location1));
        let parent_idx = storage.entities().parent_idx(entity2_idx);
        assert_eq!(parent_idx, Some(entity1_idx));
        let entity_idxs = storage.archetypes().entity_idxs(archetype2_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![entity1_idx]);
        let entity_idxs = storage.archetypes().entity_idxs(archetype3_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![entity2_idx]);
        let type_idxs = storage.archetypes().sorted_type_idxs(archetype3_idx);
        assert_eq!(type_idxs, [type1_idx, type3_idx]);
        let components = ti_vec![ti_vec![], ti_vec![10_u32], ti_vec![30_u32]];
        assert_eq!(&*storage.components().read_components::<u32>(), &components);
        let components = ti_vec![ti_vec![], ti_vec![], ti_vec![40_i8]];
        assert_eq!(&*storage.components.read_components::<i8>(), &components);
        assert_eq!(storage.components().singleton_locations(type1_idx), None);
    }

    #[test]
    fn configure_singleton_entity() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let (_, location) = storage.create_entity(archetype2_idx, None);
        storage.add_component(10_u32, type_idx, location, true);
        let singleton_location = storage.components().singleton_locations(type_idx);
        assert_eq!(singleton_location, Some(location));
    }

    #[test]
    fn run_system_that_deletes_entities() {
        let mut storage = CoreStorage::default();
        storage.create_entity_with_1_component(10_u32, None);
        storage.create_entity_with_1_component(20_u32, None);
        storage.create_entity_with_1_component(30_u32, Some(1.into()));
        storage.add_system(
            |d, i| {
                assert_eq!(i.filtered_component_type_idxs, [0.into()]);
                let mut updates = d.updates.try_lock().unwrap();
                let missing_idx = 10.into();
                updates.delete_entity(missing_idx);
                updates.delete_entity(1.into());
            },
            TypeId::of::<u32>(),
            create_system_properties(),
            None,
            ActionDependencies::Types(vec![]),
        );
        storage.update();
        let components: TiVec<_, TiVec<_, u32>> = ti_vec![ti_vec![], ti_vec![10]];
        assert_eq!(&*storage.components().read_components::<u32>(), &components);
        assert_eq!(storage.entities().location(1.into()), None);
    }

    #[test]
    fn run_system_that_updates_entities() {
        let mut storage = CoreStorage::default();
        storage.create_entity_with_1_component(10_u32, None);
        storage.register_component_type::<i64>();
        storage.add_system(
            |d, i| {
                assert_eq!(i.filtered_component_type_idxs, [0.into()]);
                let mut updates = d.updates.try_lock().unwrap();
                let entity_idx = 0.into();
                updates.delete_component(entity_idx, 0.into());
                updates.add_component(
                    entity_idx,
                    |c, a| c.add_component_type::<i64>(a).1,
                    Box::new(move |c, l| c.add_component(20_i64, 1.into(), l, false)),
                );
            },
            TypeId::of::<u32>(),
            create_system_properties(),
            None,
            ActionDependencies::Types(vec![]),
        );
        storage.update();
        let components: TiVec<_, TiVec<_, u32>> = ti_vec![ti_vec![], ti_vec![]];
        assert_eq!(&*storage.components().read_components::<u32>(), &components);
        let components: TiVec<_, TiVec<_, i64>> = ti_vec![ti_vec![], ti_vec![], ti_vec![20]];
        assert_eq!(&*storage.components().read_components::<i64>(), &components);
    }

    #[test]
    fn run_system_that_creates_entities() {
        let mut storage = CoreStorage::default();
        storage.create_entity_with_1_component(10_u32, None);
        storage.add_system(
            |d, i| {
                assert_eq!(i.filtered_component_type_idxs, [0.into()]);
                let mut updates = d.updates.try_lock().unwrap();
                updates.create_entity(
                    None,
                    Box::new(|c| {
                        c.create_entity_with_1_component(20_u32, None);
                    }),
                );
                updates.create_entity(
                    Some(0.into()),
                    Box::new(|c| {
                        c.create_entity_with_1_component(30_u32, Some(0.into()));
                    }),
                );
            },
            TypeId::of::<u32>(),
            create_system_properties(),
            None,
            ActionDependencies::Types(vec![]),
        );
        storage.update();
        let components: TiVec<_, TiVec<_, u32>> = ti_vec![ti_vec![], ti_vec![10, 30, 20]];
        assert_eq!(&*storage.components().read_components::<u32>(), &components);
    }

    fn create_system_properties() -> SystemProperties {
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Read,
                type_idx: 0.into(),
            }],
            can_update: true,
            archetype_filter: ArchetypeFilter::None,
        }
    }
}
