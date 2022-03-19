use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, EntityLocation};
use crate::utils;
use fxhash::FxHashMap;
use std::any::{Any, TypeId};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct ComponentStorage {
    idxs: FxHashMap<TypeId, ComponentTypeIdx>,
    are_entity_types: TiVec<ComponentTypeIdx, bool>,
    archetypes: TiVec<ComponentTypeIdx, Box<dyn ComponentArchetypeLock>>,
    sorted_archetype_idxs: TiVec<ComponentTypeIdx, Vec<ArchetypeIdx>>,
    singleton_locations: TiVec<ComponentTypeIdx, Option<EntityLocation>>,
}

impl ComponentStorage {
    pub(crate) fn type_idx(&self, component_type: TypeId) -> Option<ComponentTypeIdx> {
        self.idxs.get(&component_type).copied()
    }

    pub(crate) fn sorted_archetype_idxs(&self, type_idx: ComponentTypeIdx) -> &[ArchetypeIdx] {
        &self.sorted_archetype_idxs[type_idx]
    }

    pub(crate) fn singleton_location(&self, type_idx: ComponentTypeIdx) -> Option<EntityLocation> {
        self.singleton_locations[type_idx]
    }

    pub(crate) fn is_entity_type<C>(&self) -> bool
    where
        C: Any,
    {
        self.idxs
            .get(&TypeId::of::<C>())
            .map_or(false, |&i| self.are_entity_types[i])
    }

    pub(crate) fn read_components<C>(&self) -> RwLockReadGuard<'_, ComponentArchetypes<C>>
    where
        C: Any,
    {
        let &type_idx = self
            .idxs
            .get(&TypeId::of::<C>())
            .expect("internal error: cannot read missing component type");
        self.archetypes[type_idx]
            .as_any()
            .downcast_ref::<RwLock<ComponentArchetypes<C>>>()
            .expect("internal error: wrong component type when reading components")
            .try_read()
            .expect("internal error: cannot read archetypes when reading components")
    }

    pub(crate) fn write_components<C>(&self) -> RwLockWriteGuard<'_, ComponentArchetypes<C>>
    where
        C: Any,
    {
        let &type_idx = self
            .idxs
            .get(&TypeId::of::<C>())
            .expect("internal error: cannot write missing component type");
        self.archetypes[type_idx]
            .as_any()
            .downcast_ref::<RwLock<ComponentArchetypes<C>>>()
            .expect("internal error: wrong component type when writing components")
            .try_write()
            .expect("internal error: cannot write archetypes when writing component")
    }

    pub(crate) fn type_idx_or_create<C>(&mut self) -> ComponentTypeIdx
    where
        C: Any + Sync + Send,
    {
        *self.idxs.entry(TypeId::of::<C>()).or_insert_with(|| {
            self.are_entity_types.push(false);
            let archetype_lock = RwLock::new(ComponentArchetypes::<C>::default());
            self.archetypes.push(Box::new(archetype_lock));
            self.sorted_archetype_idxs.push(vec![]);
            self.singleton_locations.push_and_get_key(None)
        })
    }

    pub(super) fn add_entity_type<C>(&mut self) -> ComponentTypeIdx
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.type_idx_or_create::<C>();
        self.are_entity_types[type_idx] = true;
        type_idx
    }

    pub(super) fn add<C>(
        &mut self,
        type_idx: ComponentTypeIdx,
        location: EntityLocation,
        component: C,
        is_singleton: bool,
    ) where
        C: Any + Send + Sync,
    {
        let archetypes = self.archetypes[type_idx]
            .as_any_mut()
            .downcast_mut::<RwLock<ComponentArchetypes<C>>>()
            .expect("internal error: wrong component type when adding component")
            .get_mut()
            .expect("internal error: cannot write archetypes when adding component");
        if let Some(archetype) = archetypes.get_mut(location.idx) {
            if let Some(existing_component) = archetype.get_mut(location.pos) {
                *existing_component = component;
            } else {
                archetype.push(component);
                self.add_archetype(type_idx, location.idx);
            }
        } else {
            utils::set_value(archetypes, location.idx, ti_vec![component]);
            self.add_archetype(type_idx, location.idx);
        }
        if is_singleton {
            self.singleton_locations[type_idx] = Some(location);
        }
    }

    pub(super) fn move_(
        &mut self,
        type_idx: ComponentTypeIdx,
        src_location: EntityLocation,
        dst_archetype_idx: ArchetypeIdx,
    ) {
        self.archetypes[type_idx].move_component(src_location, dst_archetype_idx);
        self.add_archetype(type_idx, dst_archetype_idx);
        if let Some(singleton_location) = self.singleton_locations[type_idx] {
            if singleton_location == src_location {
                let location = EntityLocation {
                    idx: dst_archetype_idx,
                    pos: ArchetypeEntityPos::default(),
                };
                self.singleton_locations[type_idx] = Some(location);
            }
        }
    }

    pub(super) fn delete(&mut self, type_idx: ComponentTypeIdx, location: EntityLocation) {
        self.archetypes[type_idx].delete_component(location);
        if let Some(singleton_location) = self.singleton_locations[type_idx] {
            if singleton_location == location {
                self.singleton_locations[type_idx] = None;
            }
        }
    }

    fn add_archetype(&mut self, type_idx: ComponentTypeIdx, archetype_idx: ArchetypeIdx) {
        if let Err(pos) = self.sorted_archetype_idxs[type_idx].binary_search(&archetype_idx) {
            self.sorted_archetype_idxs[type_idx].insert(pos, archetype_idx);
        }
    }
}

trait ComponentArchetypeLock: Any + Sync + Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn move_component(&mut self, src_location: EntityLocation, dst_archetype_idx: ArchetypeIdx);

    fn delete_component(&mut self, location: EntityLocation);
}

pub(crate) type ComponentArchetypes<C> = TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, C>>;

impl<C> ComponentArchetypeLock for RwLock<ComponentArchetypes<C>>
where
    C: Any + Sync + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn move_component(&mut self, src_location: EntityLocation, dst_archetype_idx: ArchetypeIdx) {
        let archetypes = self
            .get_mut()
            .expect("internal error: cannot write archetypes when moving component");
        let component = archetypes[src_location.idx].swap_remove(src_location.pos);
        if let Some(archetype) = archetypes.get_mut(dst_archetype_idx) {
            archetype.push(component);
        } else {
            utils::set_value(&mut *archetypes, dst_archetype_idx, ti_vec![component]);
        }
    }

    fn delete_component(&mut self, location: EntityLocation) {
        let archetypes = self
            .get_mut()
            .expect("internal error: cannot write archetypes when deleting component");
        archetypes[location.idx].swap_remove(location.pos);
    }
}

idx_type!(pub ComponentTypeIdx);

#[cfg(test)]
mod component_storage_tests {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::{ComponentArchetypes, ComponentStorage};
    use std::any::{Any, TypeId};
    use std::sync::{RwLock, RwLockWriteGuard};

    impl ComponentStorage {
        pub(crate) fn try_write_components<C>(
            &self,
        ) -> Option<RwLockWriteGuard<'_, ComponentArchetypes<C>>>
        where
            C: Any,
        {
            let &type_idx = self
                .idxs
                .get(&TypeId::of::<C>())
                .expect("internal error: cannot write missing component type");
            self.archetypes[type_idx]
                .as_any()
                .downcast_ref::<RwLock<ComponentArchetypes<C>>>()
                .expect("internal error: wrong component type when writing components")
                .try_write()
                .ok()
        }
    }

    #[test]
    fn create_component_types() {
        let mut storage = ComponentStorage::default();
        let type1_idx = storage.type_idx_or_create::<u32>();
        let type2_idx = storage.type_idx_or_create::<i64>();
        let type3_idx = storage.type_idx_or_create::<u32>();
        let entity_type_idx = storage.add_entity_type::<u16>();
        storage.add_entity_type::<u16>();
        storage.add_entity_type::<u32>();
        assert_eq!([type1_idx, type3_idx], [0.into(); 2]);
        assert_eq!(type2_idx, 1.into());
        assert_eq!(entity_type_idx, 2.into());
        assert_eq!(storage.type_idx(TypeId::of::<u32>()), Some(type1_idx));
        assert_eq!(storage.type_idx(TypeId::of::<i64>()), Some(type2_idx));
        assert_eq!(storage.type_idx(TypeId::of::<u16>()), Some(entity_type_idx));
        assert_eq!(storage.type_idx(TypeId::of::<i8>()), None);
        assert!(storage.is_entity_type::<u32>());
        assert!(!storage.is_entity_type::<i64>());
        assert!(storage.is_entity_type::<u16>());
        assert!(!storage.is_entity_type::<String>());
    }

    #[test]
    fn manage_not_singleton_components() {
        let mut storage = ComponentStorage::default();
        let type_idx = storage.type_idx_or_create::<u32>();
        storage.add_entity_type::<u32>();
        let archetype1_idx = 1.into();
        let archetype2_idx = 2.into();
        let archetype3_idx = 3.into();
        let location1 = EntityLocation::new(archetype2_idx, 0.into());
        storage.add(type_idx, location1, 10_u32, false);
        let location2 = EntityLocation::new(archetype1_idx, 0.into());
        storage.add(type_idx, location2, 20_u32, false);
        storage.add(type_idx, location2, 30_u32, false);
        let location3 = EntityLocation::new(archetype1_idx, 1.into());
        storage.add(type_idx, location3, 40_u32, false);
        let location4 = EntityLocation::new(archetype1_idx, 2.into());
        storage.add(type_idx, location4, 50_u32, false);
        let location5 = EntityLocation::new(archetype1_idx, 3.into());
        storage.add(type_idx, location5, 60_u32, false);
        storage.move_(type_idx, location2, archetype3_idx);
        assert_eq!(storage.singleton_location(type_idx), None);
        storage.delete(type_idx, location2);
        let components = ti_vec![ti_vec![], ti_vec![50_u32, 40], ti_vec![10], ti_vec![30]];
        assert_eq!(&*storage.read_components::<u32>(), &components);
        assert_eq!(&*storage.write_components::<u32>(), &components);
        let sorted_archetype_idxs = storage.sorted_archetype_idxs(type_idx);
        let expected_sorted_archetypes = [archetype1_idx, archetype2_idx, archetype3_idx];
        assert_eq!(sorted_archetype_idxs, expected_sorted_archetypes);
        assert_eq!(storage.singleton_location(type_idx), None);
    }

    #[test]
    fn manage_singleton_component() {
        let mut storage = ComponentStorage::default();
        let type_idx = storage.type_idx_or_create::<u32>();
        storage.add_entity_type::<u32>();
        let archetype1_idx = 1.into();
        let archetype2_idx = 2.into();
        let location1 = EntityLocation::new(archetype1_idx, 0.into());
        storage.add(type_idx, location1, 10_u32, true);
        assert_eq!(storage.singleton_location(type_idx), Some(location1));
        storage.move_(type_idx, location1, archetype2_idx);
        let location2 = EntityLocation::new(archetype2_idx, 0.into());
        assert_eq!(storage.singleton_location(type_idx), Some(location2));

        storage.add(type_idx, location1, 20_u32, false);
        assert_eq!(storage.singleton_location(type_idx), Some(location2));
        storage.move_(type_idx, location1, archetype2_idx);
        assert_eq!(storage.singleton_location(type_idx), Some(location2));
        let location3 = EntityLocation::new(archetype2_idx, 1.into());
        storage.delete(type_idx, location3);
        assert_eq!(storage.singleton_location(type_idx), Some(location2));

        storage.delete(type_idx, location2);
        assert_eq!(storage.singleton_location(type_idx), None);
    }
}
