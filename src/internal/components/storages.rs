use crate::internal::components::data::{ComponentReadGuard, ComponentWriteGuard};
use crate::internal::entities::data::EntityLocation;
use fxhash::{FxHashMap, FxHashSet};
use std::any::{Any, TypeId};
use std::sync::RwLock;

#[derive(Default)]
pub(super) struct EntityMainComponentTypeStorage(FxHashSet<TypeId>);

impl EntityMainComponentTypeStorage {
    pub(super) fn add(&mut self, entity_type: TypeId) -> bool {
        self.0.insert(entity_type)
    }
}

#[derive(Default)]
pub(super) struct ComponentTypeStorage(FxHashMap<TypeId, usize>);

impl ComponentTypeStorage {
    pub(super) fn idx(&self, type_id: TypeId) -> Option<usize> {
        self.0.get(&type_id).copied()
    }

    pub(super) fn add(&mut self, type_id: TypeId) -> usize {
        self.0.get(&type_id).copied().unwrap_or_else(|| {
            let type_idx = self.0.len();
            self.0.insert(type_id, type_idx);
            type_idx
        })
    }
}

#[derive(Default)]
pub(crate) struct ComponentStorage {
    components: Vec<Box<dyn ComponentArchetypes>>,
}

impl ComponentStorage {
    pub(super) fn create_type<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        self.components
            .push(Box::new(RwLock::new(Vec::<Vec<C>>::new())))
    }

    pub(super) fn delete_archetype(&mut self, type_idx: usize, archetype_idx: usize) {
        self.components[type_idx].delete_archetype(archetype_idx);
    }

    pub(crate) fn read_components<C>(&self, type_idx: usize) -> ComponentReadGuard<'_, C>
    where
        C: Any,
    {
        ComponentReadGuard::new(
            self.components[type_idx]
                .as_any()
                .downcast_ref::<RwLock<Vec<Vec<C>>>>()
                .expect("internal error: invalid component type used when reading components")
                .try_read()
                .expect("internal error: lock poisoned or already locked when reading components"),
        )
    }

    pub(crate) fn write_components<C>(&self, type_idx: usize) -> ComponentWriteGuard<'_, C>
    where
        C: Any,
    {
        ComponentWriteGuard::new(
            self.components[type_idx]
                .as_any()
                .downcast_ref::<RwLock<Vec<Vec<C>>>>()
                .expect("internal error: invalid component type used when writing components")
                .try_write()
                .expect("internal error: lock poisoned or already locked when writing components"),
        )
    }

    pub(super) fn add<C>(&mut self, type_idx: usize, archetype_idx: usize, component: C)
    where
        C: Any,
    {
        let components = self.retrieve_components_mut(type_idx);
        (components.len()..=archetype_idx).for_each(|_| components.push(Vec::new()));
        components[archetype_idx].push(component);
    }

    pub(super) fn replace<C>(&mut self, type_idx: usize, location: EntityLocation, component: C)
    where
        C: Any,
    {
        let components = self.retrieve_components_mut(type_idx);
        components[location.archetype_idx][location.entity_pos] = component;
    }

    pub(super) fn move_(
        &mut self,
        type_idx: usize,
        src_location: EntityLocation,
        dst_archetype_idx: usize,
    ) {
        self.components[type_idx].move_(src_location, dst_archetype_idx);
    }

    pub(super) fn delete(&mut self, type_idx: usize, location: EntityLocation) {
        self.components[type_idx].delete(location);
    }

    fn retrieve_components_mut<C>(&mut self, type_idx: usize) -> &mut Vec<Vec<C>>
    where
        C: Any,
    {
        self.components[type_idx]
            .as_any_mut()
            .downcast_mut::<RwLock<Vec<Vec<C>>>>()
            .expect("internal error: invalid component type used when adding component")
            .get_mut()
            .expect("internal error: lock poisoned when adding component")
    }
}

trait ComponentArchetypes: Any + Sync + Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn delete_archetype(&mut self, archetype_idx: usize);

    fn move_(&mut self, src_location: EntityLocation, dst_archetype_idx: usize);

    fn delete(&mut self, location: EntityLocation);
}

impl<C> ComponentArchetypes for RwLock<Vec<Vec<C>>>
where
    C: Any + Sync + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn delete_archetype(&mut self, archetype_idx: usize) {
        let components = self
            .get_mut()
            .expect("internal error: lock poisoned when cleaning component archetype");
        if let Some(components) = components.get_mut(archetype_idx) {
            *components = Vec::new();
        }
    }

    fn move_(&mut self, src_location: EntityLocation, dst_archetype_idx: usize) {
        let components = self
            .get_mut()
            .expect("internal error: lock poisoned when moving component");
        (components.len()..=dst_archetype_idx).for_each(|_| components.push(Vec::new()));
        let component = components[src_location.archetype_idx].swap_remove(src_location.entity_pos);
        components[dst_archetype_idx].push(component);
    }

    fn delete(&mut self, location: EntityLocation) {
        let components = self
            .get_mut()
            .expect("internal error: lock poisoned when deleting component");
        components[location.archetype_idx].swap_remove(location.entity_pos);
    }
}

#[cfg(test)]
mod entity_type_storage_tests {
    use super::*;

    #[test]
    fn add_first_type() {
        let mut storage = EntityMainComponentTypeStorage::default();

        let is_new = storage.add(TypeId::of::<usize>());

        assert!(is_new);
    }

    #[test]
    fn add_different_type() {
        let mut storage = EntityMainComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let is_new = storage.add(TypeId::of::<i64>());

        assert!(is_new);
    }

    #[test]
    fn add_same_type() {
        let mut storage = EntityMainComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let is_new = storage.add(TypeId::of::<u32>());

        assert!(!is_new);
    }
}

#[cfg(test)]
mod component_type_storage_tests {
    use super::*;

    #[test]
    fn add_first_type() {
        let mut storage = ComponentTypeStorage::default();

        let type_idx = storage.add(TypeId::of::<u32>());

        assert_eq!(type_idx, 0);
        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), None);
    }

    #[test]
    fn add_different_type() {
        let mut storage = ComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let type_idx = storage.add(TypeId::of::<i64>());

        assert_eq!(type_idx, 1);
        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), Some(1));
        assert_eq!(storage.idx(TypeId::of::<String>()), None);
    }

    #[test]
    fn add_same_type() {
        let mut storage = ComponentTypeStorage::default();
        storage.add(TypeId::of::<u32>());

        let type_idx = storage.add(TypeId::of::<u32>());

        assert_eq!(type_idx, 0);
        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), None);
    }
}

#[cfg(test)]
mod component_facade_tests {
    use super::*;

    #[test]
    fn create_types() {
        let mut storage = ComponentStorage::default();

        storage.create_type::<u32>();
        storage.create_type::<i64>();

        assert_panics!(storage.read_components::<i64>(0));
        assert_panics!(storage.read_components::<u32>(1));
        assert_panics!(storage.read_components::<u32>(2));
        storage.read_components::<u32>(0);
        storage.read_components::<i64>(1);
    }

    #[test]
    #[should_panic]
    fn add_component_with_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.add(0, 0, 10_u32);
    }

    #[test]
    #[should_panic]
    fn add_component_with_invalid_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();

        storage.add(1, 0, 10_i64);
    }

    #[test]
    fn add_component_with_valid_type_in_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();

        storage.add(1, 2, 10_u32);

        let components = storage.read_components::<u32>(1);
        assert_option_iter!(components.archetype_iter(0), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(1), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(2), Some(vec![&10]));
        assert_option_iter!(components.archetype_iter(3), None);
    }

    #[test]
    fn add_component_with_valid_type_in_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        storage.add(1, 2, 20_u32);

        let components = storage.read_components::<u32>(1);
        assert_option_iter!(components.archetype_iter(0), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(1), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(2), Some(vec![&10, &20]));
        assert_option_iter!(components.archetype_iter(3), None);
    }

    #[test]
    #[should_panic]
    fn delete_archetype_with_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.delete_archetype(0, 0);
    }

    #[test]
    fn delete_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.delete_archetype(0, 0);
        let components = storage.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(0), None);
    }

    #[test]
    fn delete_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        storage.delete_archetype(1, 2);

        let components = storage.read_components::<u32>(1);
        assert_option_iter!(components.archetype_iter(0), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(1), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(2), Some(Vec::new()));
        assert_option_iter!(components.archetype_iter(3), None);
    }

    #[test]
    #[should_panic]
    fn read_components_with_missing_type() {
        let storage = ComponentStorage::default();

        storage.read_components::<u32>(0);
    }

    #[test]
    #[should_panic]
    fn read_components_with_wrong_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();

        storage.read_components::<u32>(0);
    }

    #[test]
    #[should_panic]
    fn read_components_already_written() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        let _guard = storage.write_components::<u32>(0);

        storage.read_components::<u32>(0);
    }

    #[test]
    fn read_components_already_read() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add(0, 0, 10_u32);
        let _guard = storage.read_components::<u32>(0);

        let other_guard = storage.read_components::<u32>(0);

        assert_option_iter!(other_guard.archetype_iter(0), Some(vec![&10]));
    }

    #[test]
    fn read_components_with_valid_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        let guard = storage.read_components::<u32>(1);

        assert_option_iter!(guard.archetype_iter(0), Some(Vec::new()));
        assert_option_iter!(guard.archetype_iter(1), Some(Vec::new()));
        assert_option_iter!(guard.archetype_iter(2), Some(vec![&10]));
        assert_option_iter!(guard.archetype_iter(3), None);
    }

    #[test]
    #[should_panic]
    fn write_components_with_missing_type() {
        let storage = ComponentStorage::default();

        storage.write_components::<u32>(0);
    }

    #[test]
    #[should_panic]
    fn write_components_with_wrong_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();

        storage.write_components::<u32>(0);
    }

    #[test]
    #[should_panic]
    fn write_components_already_locked() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        let _guard = storage.write_components::<u32>(0);

        storage.write_components::<u32>(0);
    }

    #[test]
    fn write_components_with_valid_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        let mut guard = storage.write_components::<u32>(1);

        assert_option_iter!(guard.archetype_iter_mut(0), Some(Vec::new()));
        assert_option_iter!(guard.archetype_iter_mut(1), Some(Vec::new()));
        assert_option_iter!(guard.archetype_iter_mut(2), Some(vec![&mut 10]));
        assert_option_iter!(guard.archetype_iter_mut(3), None);
    }

    #[test]
    #[should_panic]
    fn replace_component_with_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.replace(0, EntityLocation::new(0, 0), 10_u32);
    }

    #[test]
    #[should_panic]
    fn replace_component_with_invalid_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();

        storage.replace(0, EntityLocation::new(0, 0), 10_u32);
    }

    #[test]
    #[should_panic]
    fn replace_component_in_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();

        storage.replace(1, EntityLocation::new(0, 0), 10_u32);
    }

    #[test]
    #[should_panic]
    fn replace_component_in_missing_position() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        storage.replace(1, EntityLocation::new(2, 3), 20_u32);
    }

    #[test]
    fn replace_component_in_existing_position() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);
        storage.add(1, 2, 20_u32);
        storage.add(1, 2, 30_u32);

        storage.replace(1, EntityLocation::new(2, 0), 50_u32);

        let guard = storage.read_components::<u32>(1);
        assert_option_iter!(guard.archetype_iter(2), Some(vec![&50, &20, &30]));
    }

    #[test]
    #[should_panic]
    fn move_component_with_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.move_(0, EntityLocation::new(0, 0), 1);
    }

    #[test]
    #[should_panic]
    fn move_component_with_invalid_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();

        storage.move_(0, EntityLocation::new(0, 0), 1);
    }

    #[test]
    #[should_panic]
    fn move_component_from_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();

        storage.move_(1, EntityLocation::new(0, 0), 1);
    }

    #[test]
    #[should_panic]
    fn move_component_from_missing_position() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        storage.move_(1, EntityLocation::new(0, 1), 1);
    }

    #[test]
    fn move_component_from_existing_position() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);
        storage.add(1, 2, 20_u32);
        storage.add(1, 2, 30_u32);

        storage.move_(1, EntityLocation::new(2, 0), 3);

        let guard = storage.read_components::<u32>(1);
        assert_option_iter!(guard.archetype_iter(2), Some(vec![&30, &20]));
        assert_option_iter!(guard.archetype_iter(3), Some(vec![&10]));
    }

    #[test]
    #[should_panic]
    fn delete_component_with_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.delete(0, EntityLocation::new(0, 0));
    }

    #[test]
    #[should_panic]
    fn delete_component_with_invalid_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();

        storage.delete(0, EntityLocation::new(0, 0));
    }

    #[test]
    #[should_panic]
    fn delete_component_from_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();

        storage.delete(1, EntityLocation::new(0, 0));
    }

    #[test]
    #[should_panic]
    fn delete_component_from_missing_position() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);

        storage.delete(1, EntityLocation::new(0, 1));
    }

    #[test]
    fn delete_component_from_existing_position() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();
        storage.add(1, 2, 10_u32);
        storage.add(1, 2, 20_u32);
        storage.add(1, 2, 30_u32);

        storage.delete(1, EntityLocation::new(2, 0));

        let guard = storage.read_components::<u32>(1);
        assert_option_iter!(guard.archetype_iter(0), Some(Vec::new()));
        assert_option_iter!(guard.archetype_iter(1), Some(Vec::new()));
        assert_option_iter!(guard.archetype_iter(2), Some(vec![&30, &20]));
        assert_option_iter!(guard.archetype_iter(3), None);
    }
}
