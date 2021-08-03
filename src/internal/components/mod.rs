use crate::internal::components::data::{ComponentReadGuard, ComponentWriteGuard};
use crate::internal::components::storages::{
    ComponentStorage, ComponentTypeStorage, EntityMainComponentTypeStorage,
};
use crate::internal::entities::data::EntityLocation;
use std::any::{Any, TypeId};

// TODO: test

pub(crate) mod data;
mod storages;

#[derive(Default)]
pub(crate) struct ComponentFacade {
    entity_main_component_types: EntityMainComponentTypeStorage,
    component_types: ComponentTypeStorage,
    components: ComponentStorage,
}

impl ComponentFacade {
    pub(crate) fn read_components<C>(&self) -> Option<ComponentReadGuard<'_, C>>
    where
        C: Any,
    {
        let type_idx = self.component_types.idx(TypeId::of::<C>())?;
        Some(self.components.read_components(type_idx))
    }

    pub(crate) fn write_components<C>(&self) -> Option<ComponentWriteGuard<'_, C>>
    where
        C: Any,
    {
        let type_idx = self.component_types.idx(TypeId::of::<C>())?;
        Some(self.components.write_components(type_idx))
    }

    pub(super) fn type_idx(&self, type_id: TypeId) -> Option<usize> {
        self.component_types.idx(type_id)
    }

    pub(super) fn type_idx_or_create<C>(&mut self) -> usize
    where
        C: Any + Sync + Send,
    {
        self.component_types
            .idx(TypeId::of::<C>())
            .unwrap_or_else(|| {
                self.components.create_type::<C>();
                self.component_types.add(TypeId::of::<C>())
            })
    }

    pub(super) fn add_entity_main_component_type(&mut self, type_idx: TypeId) -> bool {
        self.entity_main_component_types.add(type_idx)
    }

    pub(super) fn delete_archetype(&mut self, type_idx: usize, archetype_idx: usize) {
        self.components.delete_archetype(type_idx, archetype_idx);
    }

    pub(super) fn delete_entity(&mut self, type_idxs: &[usize], location: EntityLocation) {
        for &type_idx in type_idxs {
            self.components.delete(type_idx, location);
        }
    }

    pub(super) fn add<C>(
        &mut self,
        moved_type_idxs: &[usize],
        location: Option<EntityLocation>,
        new_archetype_idx: usize,
        type_idx: usize,
        component: C,
    ) where
        C: Any,
    {
        if let Some(location) = location {
            for &moved_type_idx in moved_type_idxs {
                self.components
                    .move_(moved_type_idx, location, new_archetype_idx);
            }
        }
        self.components.add(type_idx, new_archetype_idx, component)
    }

    pub(super) fn replace<C>(&mut self, type_idx: usize, location: EntityLocation, component: C)
    where
        C: Any,
    {
        self.components.replace(type_idx, location, component);
    }

    pub(super) fn delete(
        &mut self,
        moved_type_idxs: &[usize],
        location: EntityLocation,
        new_archetype_idx: Option<usize>,
        type_idx: usize,
    ) {
        self.components.delete(type_idx, location);
        for &moved_type_idx in moved_type_idxs {
            if moved_type_idx != type_idx {
                if let Some(new_archetype_idx) = new_archetype_idx {
                    self.components
                        .move_(moved_type_idx, location, new_archetype_idx);
                } else {
                    self.components.delete(moved_type_idx, location);
                }
            }
        }
    }
}

#[cfg(test)]
mod component_facade_tests {
    use super::*;

    #[test]
    fn add_entity_main_component_type() {
        let mut facade = ComponentFacade::default();

        facade.add_entity_main_component_type(TypeId::of::<u32>());

        assert!(!facade.entity_main_component_types.add(TypeId::of::<u32>()));
    }

    #[test]
    fn retrieve_type_idx_or_create() {
        let mut facade = ComponentFacade::default();

        let type_idx = facade.type_idx_or_create::<u32>();

        assert_eq!(type_idx, 0);
        assert_eq!(facade.component_types.add(TypeId::of::<u32>()), 0);
    }

    #[test]
    fn add_first_components() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();

        facade.add(&[], None, 2, 0, 10_u32);

        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(2), Some(vec![&10]));
    }

    #[test]
    fn add_other_components() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.type_idx_or_create::<i64>();
        facade.add(&[], None, 2, 0, 10_u32);

        facade.add(&[0], Some(EntityLocation::new(2, 0)), 3, 1, 20_i64);

        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(2), Some(vec![]));
        assert_option_iter!(components.archetype_iter(3), Some(vec![&10]));
        let components = facade.components.read_components::<i64>(1);
        assert_option_iter!(components.archetype_iter(3), Some(vec![&20]));
    }

    #[test]
    fn write_components() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.type_idx_or_create::<i64>();
        facade.add(&[], None, 2, 0, 10_u32);

        let mut components = facade.components.write_components::<u32>(0);

        assert_option_iter!(components.archetype_iter_mut(2), Some(vec![&mut 10]));
    }

    #[test]
    fn type_idx() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.type_idx_or_create::<i64>();

        let type_idx = facade.type_idx(TypeId::of::<i64>());

        assert_eq!(type_idx, Some(1));
    }

    #[test]
    fn delete_archetype() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.add(&[], None, 2, 0, 10_u32);
        facade.add(&[], None, 3, 0, 20_u32);

        facade.delete_archetype(0, 2);

        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(2), Some(vec![]));
        assert_option_iter!(components.archetype_iter(3), Some(vec![&20]));
    }

    #[test]
    fn delete_entity() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.add(&[], None, 2, 0, 10_u32);
        facade.add(&[], None, 2, 0, 20_u32);

        facade.delete_entity(&[0], EntityLocation::new(2, 0));

        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(2), Some(vec![&20]));
    }

    #[test]
    fn replace_component() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.add(&[], None, 2, 0, 10_u32);
        facade.add(&[], None, 2, 0, 20_u32);

        facade.replace(0, EntityLocation::new(2, 1), 30_u32);
        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(2), Some(vec![&10, &30]));
    }

    #[test]
    fn delete_component_with_new_archetype_specified() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.type_idx_or_create::<i64>();
        facade.add(&[], None, 2, 0, 10_u32);
        facade.add(&[0], Some(EntityLocation::new(2, 0)), 3, 1, 20_i64);

        facade.delete(&[0, 1], EntityLocation::new(3, 0), Some(1), 0);

        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        assert_option_iter!(components.archetype_iter(3), Some(vec![]));
        let components = facade.components.read_components::<i64>(1);
        assert_option_iter!(components.archetype_iter(1), Some(vec![&20]));
        assert_option_iter!(components.archetype_iter(3), Some(vec![]));
    }

    #[test]
    fn delete_component_with_no_new_archetype_specified() {
        let mut facade = ComponentFacade::default();
        facade.type_idx_or_create::<u32>();
        facade.type_idx_or_create::<i64>();
        facade.add(&[], None, 2, 0, 10_u32);
        facade.add(&[0], Some(EntityLocation::new(2, 0)), 3, 1, 20_i64);

        facade.delete(&[0, 1], EntityLocation::new(3, 0), None, 0);

        let components = facade.components.read_components::<u32>(0);
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        assert_option_iter!(components.archetype_iter(2), Some(vec![]));
        assert_option_iter!(components.archetype_iter(3), Some(vec![]));
        let components = facade.components.read_components::<i64>(1);
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        assert_option_iter!(components.archetype_iter(2), Some(vec![]));
        assert_option_iter!(components.archetype_iter(3), Some(vec![]));
    }
}
