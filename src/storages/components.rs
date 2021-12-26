use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, EntityLocationInArchetype};
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
    component_count: TiVec<ComponentTypeIdx, usize>,
    sorted_archetype_idxs: TiVec<ComponentTypeIdx, Vec<ArchetypeIdx>>,
}

impl ComponentStorage {
    pub(crate) fn type_idx(&self, component_type: TypeId) -> Option<ComponentTypeIdx> {
        self.idxs.get(&component_type).copied()
    }

    pub(crate) fn sorted_archetype_idxs(&self, type_idx: ComponentTypeIdx) -> &[ArchetypeIdx] {
        &self.sorted_archetype_idxs[type_idx]
    }

    pub(crate) fn is_entity_type<C>(&self) -> bool
    where
        C: Any,
    {
        self.idxs
            .get(&TypeId::of::<C>())
            .map_or(false, |&i| self.are_entity_types[i])
    }

    pub(crate) fn count(&self, type_idx: ComponentTypeIdx) -> usize {
        self.component_count.get(type_idx).copied().unwrap_or(0)
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
            self.sorted_archetype_idxs.push_and_get_key(vec![])
        })
    }

    pub(super) fn add_entity_type<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.type_idx_or_create::<C>();
        self.are_entity_types[type_idx] = true;
    }

    pub(super) fn add<C>(
        &mut self,
        type_idx: ComponentTypeIdx,
        location: EntityLocationInArchetype,
        component: C,
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
                self.component_count[type_idx] += 1;
                if let Err(pos) = self.sorted_archetype_idxs[type_idx].binary_search(&location.idx)
                {
                    self.sorted_archetype_idxs[type_idx].insert(pos, location.idx);
                }
            }
        } else {
            utils::set_value(archetypes, location.idx, ti_vec![component]);
            utils::set_value(&mut self.component_count, type_idx, 1);
            if let Err(pos) = self.sorted_archetype_idxs[type_idx].binary_search(&location.idx) {
                self.sorted_archetype_idxs[type_idx].insert(pos, location.idx);
            }
        }
    }

    pub(super) fn move_(
        &mut self,
        type_idx: ComponentTypeIdx,
        src_location: EntityLocationInArchetype,
        dst_archetype_idx: ArchetypeIdx,
    ) {
        self.archetypes[type_idx].move_component(src_location, dst_archetype_idx);
    }

    pub(super) fn delete(
        &mut self,
        type_idx: ComponentTypeIdx,
        location: EntityLocationInArchetype,
    ) {
        self.archetypes[type_idx].delete_component(location);
        self.component_count[type_idx] -= 1;
    }
}

trait ComponentArchetypeLock: Any + Sync + Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn move_component(
        &mut self,
        src_location: EntityLocationInArchetype,
        dst_archetype_idx: ArchetypeIdx,
    );

    fn delete_component(&mut self, location: EntityLocationInArchetype);
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

    fn move_component(
        &mut self,
        src_location: EntityLocationInArchetype,
        dst_archetype_idx: ArchetypeIdx,
    ) {
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

    fn delete_component(&mut self, location: EntityLocationInArchetype) {
        let archetypes = self
            .get_mut()
            .expect("internal error: cannot write archetypes when deleting component");
        archetypes[location.idx].swap_remove(location.pos);
    }
}

idx_type!(pub ComponentTypeIdx);

#[cfg(test)]
mod component_storage_tests {
    use super::*;

    #[test]
    fn create_type_idxs() {
        let mut storage = ComponentStorage::default();

        let type1_idx = storage.type_idx_or_create::<u32>();
        let type2_idx = storage.type_idx_or_create::<i64>();
        let type3_idx = storage.type_idx_or_create::<u32>();

        let type1 = TypeId::of::<u32>();
        let type2 = TypeId::of::<i64>();
        let type3 = TypeId::of::<u8>();
        assert_eq!(type1_idx, 0.into());
        assert_eq!(type2_idx, 1.into());
        assert_eq!(type3_idx, 0.into());
        assert_eq!(storage.type_idx(type1), Some(type1_idx));
        assert_eq!(storage.type_idx(type2), Some(type2_idx));
        assert_eq!(storage.type_idx(type3), None);
    }

    #[test]
    fn add_entity_types() {
        let mut storage = ComponentStorage::default();
        storage.type_idx_or_create::<u32>();
        storage.type_idx_or_create::<i8>();

        storage.add_entity_type::<u32>();
        storage.add_entity_type::<i64>();

        assert_eq!(storage.type_idx(TypeId::of::<i64>()), Some(2.into()));
        assert!(storage.is_entity_type::<u32>());
        assert!(storage.is_entity_type::<i64>());
        assert!(!storage.is_entity_type::<i8>());
        assert!(!storage.is_entity_type::<u8>());
    }

    #[test]
    fn add_components() {
        let mut storage = ComponentStorage::default();
        let type_idx = storage.type_idx_or_create::<u32>();
        let location1 = EntityLocationInArchetype::new(2.into(), 0.into());
        let location2 = EntityLocationInArchetype::new(1.into(), 0.into());
        let location3 = EntityLocationInArchetype::new(1.into(), 1.into());

        storage.add(type_idx, location1, 10_u32);
        storage.add(type_idx, location2, 20_u32);
        storage.add(type_idx, location3, 30_u32);

        assert_eq!(storage.count(type_idx), 3);
        let components = ti_vec![ti_vec![], ti_vec![20_u32, 30_u32], ti_vec![10_u32]];
        assert_eq!(&*storage.read_components::<u32>(), &components);
        assert_eq!(&*storage.write_components::<u32>(), &components);
        assert_eq!(
            storage.sorted_archetype_idxs(0.into()),
            [1.into(), 2.into()]
        );
    }

    #[test]
    fn replace_components() {
        let mut storage = ComponentStorage::default();
        let type_idx = storage.type_idx_or_create::<u32>();
        let location = EntityLocationInArchetype::new(1.into(), 0.into());
        storage.add(type_idx, location, 10_u32);

        storage.add(type_idx, location, 20_u32);

        assert_eq!(storage.count(type_idx), 1);
        let components = ti_vec![ti_vec![], ti_vec![20_u32]];
        assert_eq!(&*storage.read_components::<u32>(), &components);
        assert_eq!(&*storage.write_components::<u32>(), &components);
    }

    #[test]
    fn move_components() {
        let mut storage = ComponentStorage::default();
        let type_idx = storage.type_idx_or_create::<u32>();
        let location1 = EntityLocationInArchetype::new(1.into(), 0.into());
        storage.add(type_idx, location1, 10_u32);
        let location2 = EntityLocationInArchetype::new(1.into(), 1.into());
        storage.add(type_idx, location2, 20_u32);
        let location3 = EntityLocationInArchetype::new(1.into(), 2.into());
        storage.add(type_idx, location3, 30_u32);
        let location4 = EntityLocationInArchetype::new(1.into(), 3.into());
        storage.add(type_idx, location4, 40_u32);

        storage.move_(type_idx, location1, 2.into());
        storage.move_(type_idx, location2, 2.into());

        assert_eq!(storage.count(type_idx), 4);
        let components = ti_vec![ti_vec![], ti_vec![40_u32, 30_u32], ti_vec![10_u32, 20_u32]];
        assert_eq!(&*storage.read_components::<u32>(), &components);
    }

    #[test]
    fn delete_component() {
        let mut storage = ComponentStorage::default();
        let type_idx = storage.type_idx_or_create::<u32>();
        let location1 = EntityLocationInArchetype::new(1.into(), 0.into());
        storage.add(type_idx, location1, 10_u32);
        let location2 = EntityLocationInArchetype::new(1.into(), 1.into());
        storage.add(type_idx, location2, 20_u32);
        let location3 = EntityLocationInArchetype::new(1.into(), 2.into());
        storage.add(type_idx, location3, 30_u32);

        storage.delete(type_idx, location1);

        assert_eq!(storage.count(type_idx), 2);
        let components = ti_vec![ti_vec![], ti_vec![30_u32, 20_u32]];
        assert_eq!(&*storage.read_components::<u32>(), &components);
    }
}
