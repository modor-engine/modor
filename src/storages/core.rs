use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocationInArchetype};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::entities::{EntityIdx, EntityStorage};
use crate::storages::entity_actions::{EntityActionStorage, EntityState};
use crate::storages::systems::{SystemProperties, SystemStorage};
use crate::systems::internal::SystemWrapper;
use crate::SystemData;
use std::any::{Any, TypeId};
use std::mem;
use std::sync::Mutex;

#[derive(Default)]
pub struct CoreStorage {
    archetypes: ArchetypeStorage,
    entities: EntityStorage,
    components: ComponentStorage,
    systems: SystemStorage,
    entity_actions: Mutex<EntityActionStorage>,
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
    ) -> EntityLocationInArchetype {
        let location = EntityLocationInArchetype {
            idx: archetype_idx,
            pos: self.archetypes.next_entity_pos(archetype_idx),
        };
        let entity_idx = self.entities.create(location);
        EntityLocationInArchetype {
            idx: archetype_idx,
            pos: self.archetypes.add_entity(entity_idx, archetype_idx),
        }
    }

    pub(crate) fn add_component<C>(
        &mut self,
        component: C,
        type_idx: ComponentTypeIdx,
        location: EntityLocationInArchetype,
    ) where
        C: Any + Sync + Send,
    {
        self.components.add(type_idx, location, component);
    }

    pub(crate) fn move_entity(
        &mut self,
        src_location: EntityLocationInArchetype,
        dst_archetype_idx: ArchetypeIdx,
    ) -> EntityLocationInArchetype {
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
        let dst_location = EntityLocationInArchetype {
            idx: dst_archetype_idx,
            pos: self.archetypes.add_entity(entity_idx, dst_archetype_idx),
        };
        self.entities.set_location(entity_idx, dst_location);
        self.update_moved_entity_location(src_location);
        dst_location
    }

    pub(crate) fn add_system(
        &mut self,
        wrapper: SystemWrapper,
        entity_type: TypeId,
        properties: SystemProperties,
    ) {
        let entity_type_idx = self
            .components
            .type_idx(entity_type)
            .expect("internal error: missing entity type when adding system");
        self.systems.add(wrapper, entity_type_idx, properties);
    }

    pub(crate) fn update(&mut self) {
        let data = SystemData {
            components: &self.components,
            archetypes: &self.archetypes,
            entity_actions: &self.entity_actions,
        };
        self.systems.run(data);
    }

    pub(crate) fn apply_system_actions(&mut self) {
        let mut entity_actions = mem::take(
            self.entity_actions
                .get_mut()
                .expect("internal error: cannot access to entity actions"),
        );
        for (entity_idx, entity_state) in entity_actions.drain_entity_states() {
            let location = if let Some(location) = self.entities.location(entity_idx) {
                location
            } else {
                continue;
            };
            match entity_state {
                EntityState::Unchanged(add_component_fns, deleted_component_type_idxs) => {
                    let mut dst_archetype_idx = location.idx;
                    for add_fns in &add_component_fns {
                        dst_archetype_idx = (add_fns.add_type_fn)(self, dst_archetype_idx);
                    }
                    for type_idx in deleted_component_type_idxs {
                        dst_archetype_idx = self.delete_component_type(type_idx, dst_archetype_idx);
                    }
                    let dst_location = self.move_entity(location, dst_archetype_idx);
                    for add_fns in add_component_fns {
                        (add_fns.add_fn)(self, dst_location);
                    }
                }
                EntityState::Deleted => self.delete_entity(entity_idx, location),
            }
        }
    }

    fn delete_entity(&mut self, entity_idx: EntityIdx, location: EntityLocationInArchetype) {
        for &type_idx in self.archetypes.sorted_type_idxs(location.idx) {
            self.components.delete(type_idx, location);
        }
        self.archetypes.delete_entity(location);
        self.entities.delete(entity_idx);
        self.update_moved_entity_location(location);
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

    fn update_moved_entity_location(&mut self, location: EntityLocationInArchetype) {
        let archetype_entity_idxs = self.archetypes.entity_idxs(location.idx);
        if let Some(&moved_entity_idx) = archetype_entity_idxs.get(location.pos) {
            self.entities.set_location(moved_entity_idx, location);
        }
    }
}

#[cfg(test)]
mod core_storage_tests {
    use super::*;
    use crate::storages::systems::{Access, ComponentTypeAccess};
    use crate::SystemData;
    use typed_index_collections::TiVec;
    use crate::storages::archetypes::ArchetypeFilter;

    #[allow(clippy::multiple_inherent_impl)]
    impl CoreStorage {
        pub(crate) fn system_data(&self) -> SystemData<'_> {
            SystemData {
                components: &self.components,
                archetypes: &self.archetypes,
                entity_actions: &self.entity_actions,
            }
        }
    }

    #[test]
    fn set_thread_count() {
        let mut storage = CoreStorage::default();

        storage.set_thread_count(3);

        assert_eq!(storage.systems.thread_count(), 3);
    }

    #[test]
    fn register_component_types() {
        let mut storage = CoreStorage::default();

        let type1_idx = storage.register_component_type::<u32>();
        let type2_idx = storage.register_component_type::<i64>();

        assert_eq!(type1_idx, 0.into());
        assert_eq!(type2_idx, 1.into());
        let type_id = TypeId::of::<u32>();
        assert_eq!(storage.components.type_idx(type_id), Some(0.into()));
        let type_id = TypeId::of::<i64>();
        assert_eq!(storage.components.type_idx(type_id), Some(1.into()));
    }

    #[test]
    fn add_entity_type() {
        let mut storage = CoreStorage::default();

        storage.add_entity_type::<u32>();

        assert!(storage.components.is_entity_type::<u32>());
    }

    #[test]
    fn add_first_component_type_to_archetype() {
        let mut storage = CoreStorage::default();
        let archetype_idx = ArchetypeStorage::DEFAULT_IDX;

        let (type_idx, archetype_idx) = storage.add_component_type::<u32>(archetype_idx);

        assert_eq!(type_idx, 0.into());
        assert_eq!(archetype_idx, 1.into());
        let type_idxs = storage.archetypes.sorted_type_idxs(archetype_idx);
        assert_eq!(type_idxs, [type_idx]);
    }

    #[test]
    fn add_missing_component_type_to_archetype() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);

        let (type2_idx, archetype3_idx) = storage.add_component_type::<i64>(archetype2_idx);

        assert_eq!(type2_idx, 1.into());
        assert_eq!(archetype3_idx, 2.into());
        let type_idxs = storage.archetypes.sorted_type_idxs(archetype3_idx);
        assert_eq!(type_idxs, [type1_idx, type2_idx]);
    }

    #[test]
    fn add_existing_component_type_to_archetype() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);

        let (type2_idx, archetype3_idx) = storage.add_component_type::<u32>(archetype2_idx);

        assert_eq!(type2_idx, type1_idx);
        assert_eq!(archetype3_idx, archetype2_idx);
    }

    #[test]
    fn create_entity() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (_, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);

        let location = storage.create_entity(archetype2_idx);

        assert_eq!(location.idx, archetype2_idx);
        assert_eq!(location.pos, 0.into());
        assert_eq!(storage.entities.location(0.into()), Some(location));
        let entity_idxs = storage.archetypes.entity_idxs(archetype2_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![0.into()]);
    }

    #[test]
    fn add_component() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let location = storage.create_entity(archetype2_idx);

        storage.add_component(10_u32, type_idx, location);

        let components = ti_vec![ti_vec![], ti_vec![10_u32]];
        assert_eq!(&*storage.components.read_components::<u32>(), &components);
    }

    #[test]
    fn move_entity() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let (type2_idx, archetype3_idx) = storage.add_component_type::<i64>(archetype2_idx);
        let location1 = storage.create_entity(archetype3_idx);
        storage.add_component(10_u32, type1_idx, location1);
        storage.add_component(20_i64, type2_idx, location1);
        let location2 = storage.create_entity(archetype3_idx);
        storage.add_component(30_u32, type1_idx, location2);
        storage.add_component(40_i64, type2_idx, location2);

        let location = storage.move_entity(location1, archetype2_idx);

        assert_eq!(location.idx, archetype2_idx);
        assert_eq!(location.pos, 0.into());
        assert_eq!(storage.entities.location(0.into()), Some(location));
        assert_eq!(storage.entities.location(1.into()), Some(location1));
        let entity_idxs = storage.archetypes.entity_idxs(archetype2_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![0.into()]);
        let entity_idxs = storage.archetypes.entity_idxs(archetype3_idx).to_vec();
        assert_eq!(entity_idxs, ti_vec![1.into()]);
        let components = ti_vec![ti_vec![], ti_vec![10_u32], ti_vec![30_u32]];
        assert_eq!(&*storage.components.read_components::<u32>(), &components);
        let components = ti_vec![ti_vec![], ti_vec![], ti_vec![40_i64]];
        assert_eq!(&*storage.components.read_components::<i64>(), &components);
    }

    #[test]
    fn add_system_and_update() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let location = storage.create_entity(archetype2_idx);
        storage.add_component(10_u32, type_idx, location);

        storage.add_system(
            |d, i| {
                assert_eq!(i.filtered_component_type_idxs, [0.into()]);
                d.entity_actions.try_lock().unwrap().delete_entity(0.into());
            },
            TypeId::of::<u32>(),
            SystemProperties {
                component_types: vec![ComponentTypeAccess {
                    access: Access::Read,
                    type_idx: 0.into(),
                }],
                has_entity_actions: true,
                archetype_filter: ArchetypeFilter::None
            },
        );
        storage.update();

        let mut entity_actions = storage.entity_actions.try_lock().unwrap();
        let entity_states: Vec<_> = entity_actions.drain_entity_states().collect();
        assert_eq!(entity_states[0].0, 0.into());
        assert!(matches!(entity_states[0].1, EntityState::Deleted));
    }

    #[test]
    fn apply_system_actions_on_missing_entity() {
        let mut storage = CoreStorage::default();
        storage
            .entity_actions
            .try_lock()
            .unwrap()
            .delete_entity(0.into());

        storage.apply_system_actions();

        assert_eq!(storage.entities.location(0.into()), None);
    }

    #[test]
    fn apply_system_actions_with_deleted_entity() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let location1 = storage.create_entity(archetype2_idx);
        storage.add_component(10_u32, type_idx, location1);
        let location2 = storage.create_entity(archetype2_idx);
        storage.add_component(20_u32, type_idx, location2);
        storage
            .entity_actions
            .try_lock()
            .unwrap()
            .delete_entity(0.into());

        storage.apply_system_actions();

        assert_eq!(storage.entities.location(0.into()), None);
        let location = EntityLocationInArchetype::new(archetype2_idx, 0.into());
        assert_eq!(storage.entities.location(1.into()), Some(location));
        let components: TiVec<_, TiVec<_, u32>> = ti_vec![ti_vec![], ti_vec![20_u32]];
        assert_eq!(&*storage.components.read_components::<u32>(), &components);
        assert_eq!(storage.archetypes.entity_idxs(0.into()).to_vec(), ti_vec![]);
        let entity_idxs = storage.archetypes.entity_idxs(1.into()).to_vec();
        assert_eq!(entity_idxs, ti_vec![1.into()]);
    }

    #[test]
    fn apply_system_actions_with_added_component() {
        let mut storage = CoreStorage::default();
        storage.create_entity(ArchetypeStorage::DEFAULT_IDX);
        storage.entity_actions.try_lock().unwrap().add_component(
            0.into(),
            |c, a| c.add_component_type::<u32>(a).1,
            Box::new(move |c, l| c.add_component(10_u32, 0.into(), l)),
        );

        storage.apply_system_actions();

        let location = EntityLocationInArchetype::new(1.into(), 0.into());
        assert_eq!(storage.entities.location(0.into()), Some(location));
        let components = ti_vec![ti_vec![], ti_vec![10_u32]];
        assert_eq!(&*storage.components.read_components::<u32>(), &components);
        assert_eq!(storage.archetypes.entity_idxs(0.into()).to_vec(), ti_vec![]);
        let entity_idxs = storage.archetypes.entity_idxs(1.into()).to_vec();
        assert_eq!(entity_idxs, ti_vec![0.into()]);
    }

    #[test]
    fn apply_system_actions_with_deleted_existing_component() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        let (type2_idx, archetype3_idx) = storage.add_component_type::<i64>(archetype2_idx);
        let location = storage.create_entity(archetype3_idx);
        storage.add_component(10_u32, type1_idx, location);
        storage.add_component(20_i64, type2_idx, location);
        storage
            .entity_actions
            .try_lock()
            .unwrap()
            .delete_component(0.into(), 1.into());

        storage.apply_system_actions();

        let location = EntityLocationInArchetype::new(1.into(), 0.into());
        assert_eq!(storage.entities.location(0.into()), Some(location));
        let components = ti_vec![ti_vec![], ti_vec![10_u32], ti_vec![]];
        assert_eq!(&*storage.components.read_components::<u32>(), &components);
        let components: TiVec<_, TiVec<_, i64>> = ti_vec![ti_vec![], ti_vec![], ti_vec![]];
        assert_eq!(&*storage.components.read_components::<i64>(), &components);
        let entity_idxs = storage.archetypes.entity_idxs(1.into()).to_vec();
        assert_eq!(entity_idxs, ti_vec![0.into()]);
    }

    #[test]
    fn apply_system_actions_with_deleted_missing_component() {
        let mut storage = CoreStorage::default();
        let archetype1_idx = ArchetypeStorage::DEFAULT_IDX;
        let (type1_idx, archetype2_idx) = storage.add_component_type::<u32>(archetype1_idx);
        storage.add_component_type::<i64>(archetype1_idx);
        let location = storage.create_entity(archetype2_idx);
        storage.add_component(10_u32, type1_idx, location);
        storage
            .entity_actions
            .try_lock()
            .unwrap()
            .delete_component(0.into(), 1.into());

        storage.apply_system_actions();

        let location = EntityLocationInArchetype::new(1.into(), 0.into());
        assert_eq!(storage.entities.location(0.into()), Some(location));
        let components = ti_vec![ti_vec![], ti_vec![10_u32]];
        assert_eq!(&*storage.components.read_components::<u32>(), &components);
        let entity_idxs = storage.archetypes.entity_idxs(1.into()).to_vec();
        assert_eq!(entity_idxs, ti_vec![0.into()]);
    }
}
