use crate::internal::components::{ArchetypePositionStorage, ComponentStorage, TypeStorage};
use std::any::{Any, TypeId};
use std::ops::Deref;
use std::slice::{Iter, IterMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub(crate) struct Components(pub(super) Box<dyn Any + Sync + Send>);

impl From<Box<dyn Any + Sync + Send>> for Components {
    fn from(components: Box<dyn Any + Sync + Send>) -> Self {
        Self(components)
    }
}

impl Into<Box<dyn Any + Sync + Send>> for Components {
    fn into(self) -> Box<dyn Any + Sync + Send> {
        self.0
    }
}

impl Components {
    pub(crate) fn iter<C>(&self, archetype_pos: usize) -> Option<Iter<'_, C>>
    where
        C: Any,
    {
        let components: &Vec<Vec<C>> = self
            .0
            .downcast_ref()
            .expect("internal error: iterate on components using wrong type");
        components.get(archetype_pos).map(|c| c.iter())
    }

    pub(super) fn iter_mut<C>(&mut self, archetype_pos: usize) -> Option<IterMut<'_, C>>
    where
        C: Any,
    {
        let components: &mut Vec<Vec<C>> = self
            .0
            .downcast_mut()
            .expect("internal error: mutably iterate on components using wrong type");
        components.get_mut(archetype_pos).map(|c| c.iter_mut())
    }
}

pub(crate) struct ComponentInterface<'a> {
    components: ComponentLocks<'a>,
    archetype_positions: &'a ArchetypePositionStorage,
    types: &'a TypeStorage,
}

impl<'a> ComponentInterface<'a> {
    pub(super) fn new(
        components: &'a mut ComponentStorage,
        archetype_positions: &'a ArchetypePositionStorage,
        types: &'a TypeStorage,
    ) -> Self {
        Self {
            components: ComponentLocks::new(components),
            archetype_positions,
            types,
        }
    }
}

impl ComponentInterface<'_> {
    pub(crate) fn read<C>(&self) -> Option<RwLockReadGuard<'_, Components>>
    where
        C: Any,
    {
        let type_idx = self.types.idx(TypeId::of::<C>())?;
        Some(
            self.components
                .get(type_idx)?
                .try_read()
                .expect("internal error: read already locked components"),
        )
    }

    pub(crate) fn write<C>(&self) -> Option<RwLockWriteGuard<'_, Components>>
    where
        C: Any,
    {
        let type_idx = self.types.idx(TypeId::of::<C>())?;
        Some(
            self.components
                .get(type_idx)?
                .try_write()
                .expect("internal error: mutably read already locked components"),
        )
    }

    pub(crate) fn iter<'a, C>(
        &self,
        guard: &'a RwLockReadGuard<'_, Components>,
        archetype_idx: usize,
    ) -> Option<Iter<'a, C>>
    where
        C: Any,
    {
        let type_idx = self
            .types
            .idx(TypeId::of::<C>())
            .expect("internal error: iterate on components with not registered type");
        let position_pos = self.archetype_positions.get(type_idx, archetype_idx)?;
        guard.iter(position_pos)
    }

    pub(crate) fn iter_mut<'a, C>(
        &self,
        guard: &'a mut RwLockWriteGuard<'_, Components>,
        archetype_idx: usize,
    ) -> Option<IterMut<'a, C>>
    where
        C: Any,
    {
        let type_idx = self
            .types
            .idx(TypeId::of::<C>())
            .expect("internal error: mutably iterate on components with not registered type");
        let position_pos = self.archetype_positions.get(type_idx, archetype_idx)?;
        guard.iter_mut(position_pos)
    }
}

struct ComponentLocks<'a> {
    components: &'a mut ComponentStorage,
    locks: Vec<RwLock<Components>>,
}

impl<'a> ComponentLocks<'a> {
    fn new(components: &'a mut ComponentStorage) -> Self {
        ComponentLocks {
            locks: components.export(),
            components,
        }
    }
}

impl Deref for ComponentLocks<'_> {
    type Target = [RwLock<Components>];

    fn deref(&self) -> &Self::Target {
        self.locks.as_slice()
    }
}

impl Drop for ComponentLocks<'_> {
    fn drop(&mut self) {
        self.components.import(&mut self.locks);
    }
}

#[cfg(test)]
mod components_tests {
    use super::*;

    #[test]
    fn from_component_box() {
        let component_box = Box::new(vec![vec![1_u32, 2], vec![3]]);

        let mut components = Components::from(component_box as Box<_>);

        assert_panics!(components.iter::<String>(0));
        assert_option_iter!(components.iter::<u32>(0), Some(vec![&1, &2]));
        assert_option_iter!(components.iter::<u32>(1), Some(vec![&3]));
        assert!(components.iter::<u32>(2).is_none());
        assert_panics!(components.iter_mut::<String>(0));
        assert_option_iter!(components.iter_mut::<u32>(0), Some(vec![&mut 1, &mut 2]));
        assert_option_iter!(components.iter_mut::<u32>(1), Some(vec![&mut 3]));
        assert!(components.iter_mut::<u32>(2).is_none());
    }

    #[test]
    fn into_component_box() {
        let components = Box::new(vec![vec![1_u32, 2], vec![3]]);
        let components = Components::from(components as Box<_>);

        let component_box: Box<dyn Any + Sync + Send> = components.into();

        let downcast_components: Option<&Vec<Vec<u32>>> = component_box.downcast_ref();
        assert_eq!(downcast_components, Some(&vec![vec![1, 2], vec![3]]))
    }
}

#[cfg(test)]
mod component_interface_tests {
    use super::*;
    use std::ptr;

    fn create_components() -> ComponentStorage {
        let mut components = ComponentStorage::default();
        components.create_type::<u32>();
        components.create_type::<i64>();
        components.add(0, 0, 10_u32);
        components.add(0, 1, 20_u32);
        components.add(0, 1, 30_u32);
        components.add(1, 0, 40_i64);
        components
    }

    fn create_archetype_positions() -> ArchetypePositionStorage {
        let mut archetype_positions = ArchetypePositionStorage::default();
        archetype_positions.create_type();
        archetype_positions.create_type();
        archetype_positions.create(0, 3);
        archetype_positions.create(0, 4);
        archetype_positions.create(1, 5);
        archetype_positions
    }

    fn create_types() -> TypeStorage {
        let mut types = TypeStorage::default();
        types.add(TypeId::of::<u32>());
        types.add(TypeId::of::<i64>());
        types
    }

    #[test]
    fn create_new_interface() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let component_ptr: *const _ = &components;

        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);

        assert_eq!(interface.components.locks.len(), 2);
        let type1_lock = interface.components.locks[0].read().unwrap();
        let type2_lock = interface.components.locks[1].read().unwrap();
        assert_option_iter!(type1_lock.iter::<u32>(0), Some(vec![&10]));
        assert_option_iter!(type1_lock.iter::<u32>(1), Some(vec![&20, &30]));
        assert_option_iter!(type1_lock.iter::<u32>(2), None);
        assert_option_iter!(type2_lock.iter::<i64>(0), Some(vec![&40]));
        assert_option_iter!(type2_lock.iter::<i64>(1), None);
        assert_eq!(interface.components.components as *const _, component_ptr);
        assert!(ptr::eq(interface.archetype_positions, &archetype_positions));
        assert!(ptr::eq(interface.types, &types));
    }

    #[test]
    fn drop_interface() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);

        drop(interface);

        let components = components.export();
        assert_eq!(components.len(), 2);
        let type1_lock = components[0].read().unwrap();
        let type2_lock = components[1].read().unwrap();
        assert_option_iter!(type1_lock.iter::<u32>(0), Some(vec![&10]));
        assert_option_iter!(type1_lock.iter::<u32>(1), Some(vec![&20, &30]));
        assert_option_iter!(type1_lock.iter::<u32>(2), None);
        assert_option_iter!(type2_lock.iter::<i64>(0), Some(vec![&40]));
        assert_option_iter!(type2_lock.iter::<i64>(1), None);
    }

    #[test]
    fn read_missing_component_type() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);

        let guard = interface.read::<String>();

        assert!(guard.is_none());
    }

    #[test]
    fn read_existing_component_type() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);

        let guard = interface.read::<u32>();

        let guard = guard.unwrap();
        assert_option_iter!(guard.iter::<u32>(0), Some(vec![&10]));
        assert_option_iter!(guard.iter::<u32>(1), Some(vec![&20, &30]));
        assert_option_iter!(guard.iter::<u32>(2), None);
    }

    #[test]
    fn write_missing_component_type() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);

        let guard = interface.write::<String>();

        assert!(guard.is_none());
    }

    #[test]
    fn write_existing_component_type() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);

        let guard = interface.write::<u32>();

        let mut guard = guard.unwrap();
        assert_option_iter!(guard.iter_mut::<u32>(0), Some(vec![&mut 10]));
        assert_option_iter!(guard.iter_mut::<u32>(1), Some(vec![&mut 20, &mut 30]));
        assert_option_iter!(guard.iter_mut::<u32>(2), None);
    }

    #[test]
    fn iter_on_missing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let guard = interface.read::<u32>().unwrap();

        let iter = interface.iter::<u32>(&guard, 9);

        assert_option_iter!(iter, None);
    }

    #[test]
    fn iter_with_wrong_existing_type_on_existing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let guard = interface.read::<u32>().unwrap();

        let iter = interface.iter::<i64>(&guard, 4);

        assert_option_iter!(iter, None);
    }

    #[test]
    #[should_panic]
    fn iter_with_missing_type_on_existing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let guard = interface.read::<u32>().unwrap();

        interface.iter::<String>(&guard, 4);
    }

    #[test]
    fn iter_with_existing_type_on_existing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let guard = interface.read::<u32>().unwrap();

        let iter = interface.iter::<u32>(&guard, 4);

        assert_option_iter!(iter, Some(vec![&20, &30]));
    }

    #[test]
    fn iter_mut_on_missing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let mut guard = interface.write::<u32>().unwrap();

        let iter = interface.iter_mut::<u32>(&mut guard, 9);

        assert_option_iter!(iter, None);
    }

    #[test]
    fn iter_mut_with_wrong_existing_type_on_existing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let mut guard = interface.write::<u32>().unwrap();

        let iter = interface.iter_mut::<i64>(&mut guard, 4);

        assert_option_iter!(iter, None);
    }

    #[test]
    fn iter_mut_with_missing_type_on_existing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let mut guard = interface.write::<u32>().unwrap();

        assert_panics!(interface.iter_mut::<String>(&mut guard, 4));
    }

    #[test]
    fn iter_mut_with_existing_type_on_existing_archetype() {
        let mut components = create_components();
        let archetype_positions = create_archetype_positions();
        let types = create_types();
        let interface = ComponentInterface::new(&mut components, &archetype_positions, &types);
        let mut guard = interface.write::<u32>().unwrap();

        let iter = interface.iter_mut::<u32>(&mut guard, 4);

        assert_option_iter!(iter, Some(vec![&mut 20, &mut 30]));
    }
}
