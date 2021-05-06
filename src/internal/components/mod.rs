use crate::internal::components::interfaces::ComponentInterface;
use crate::internal::components::storages::{
    ArchetypePositionStorage, ComponentStorage, TypeStorage,
};
use crate::internal::entities::data::EntityLocation;
use std::any::{Any, TypeId};

pub(crate) mod interfaces;
mod storages;

// TODO: store components directly in RwLock, and use get_mut to add components
// TODO: if done, move in core to avoid data redundancy (= better performance)

#[derive(Default)]
pub(super) struct ComponentFacade {
    components: ComponentStorage,
    archetype_positions: ArchetypePositionStorage,
    types: TypeStorage, // TODO: remove ?
}

impl ComponentFacade {
    pub(super) fn create_type<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        self.components.create_type::<C>();
        self.archetype_positions.create_type();
        self.types.add(TypeId::of::<C>());
    }

    pub(super) fn delete_archetype(&mut self, type_idx: usize, archetype_idx: usize) {
        if let Some(archetype_pos) = self.archetype_positions.get(type_idx, archetype_idx) {
            self.components.delete_archetype(type_idx, archetype_pos);
            self.archetype_positions.delete(type_idx, archetype_idx);
        }
    }

    pub(super) fn components(&mut self) -> ComponentInterface<'_> {
        ComponentInterface::new(&mut self.components, &self.archetype_positions, &self.types)
    }

    pub(super) fn exists<C>(&self, type_idx: usize, location: EntityLocation) -> bool
    where
        C: Any,
    {
        let archetype_idx = location.archetype_idx;
        let entity_pos = location.entity_pos;
        let archetype_pos = self.archetype_pos(type_idx, archetype_idx);
        archetype_pos.map_or(false, |a| {
            self.components.exists::<C>(type_idx, a, entity_pos)
        })
    }

    pub(super) fn add<C>(&mut self, type_idx: usize, archetype_idx: usize, component: C)
    where
        C: Any,
    {
        let archetype_pos = self.archetype_pos_or_create(type_idx, archetype_idx);
        self.components.add(type_idx, archetype_pos, component);
    }

    pub(super) fn replace<C>(&mut self, type_idx: usize, location: EntityLocation, component: C)
    where
        C: Any,
    {
        let archetype_idx = location.archetype_idx;
        let entity_pos = location.entity_pos;
        let archetype_pos = self.archetype_pos(type_idx, archetype_idx).unwrap();
        self.components
            .replace(type_idx, archetype_pos, entity_pos, component);
    }

    pub(super) fn move_(
        &mut self,
        type_idx: usize,
        src_location: EntityLocation,
        dst_archetype_idx: usize,
    ) {
        let src_archetype_idx = src_location.archetype_idx;
        let src_entity_pos = src_location.entity_pos;
        let src_archetype_pos = self.archetype_pos(type_idx, src_archetype_idx).unwrap();
        let dst_archetype_pos = self.archetype_pos_or_create(type_idx, dst_archetype_idx);
        self.components.move_(
            type_idx,
            src_archetype_pos,
            src_entity_pos,
            dst_archetype_pos,
        );
    }

    pub(super) fn swap_delete(&mut self, type_idx: usize, location: EntityLocation) {
        let archetype_idx = location.archetype_idx;
        let entity_pos = location.entity_pos;
        let archetype_pos = self.archetype_pos(type_idx, archetype_idx).unwrap();
        self.components
            .swap_delete(type_idx, archetype_pos, entity_pos);
    }

    fn archetype_pos_or_create(&mut self, type_idx: usize, archetype_idx: usize) -> usize {
        self.archetype_positions
            .get(type_idx, archetype_idx)
            .unwrap_or_else(|| self.archetype_positions.create(type_idx, archetype_idx))
    }

    fn archetype_pos(&self, type_idx: usize, archetype_idx: usize) -> Option<usize> {
        self.archetype_positions.get(type_idx, archetype_idx)
    }
}

#[cfg(test)]
mod tests_component_facade {
    use super::*;

    #[test]
    fn create_type() {
        let mut facade = ComponentFacade::default();

        facade.create_type::<u32>();

        assert_eq!(facade.components.export().len(), 1);
        assert_eq!(facade.archetype_positions.get(0, 0), None);
        assert_eq!(facade.types.idx(TypeId::of::<u32>()), Some(0));
    }

    #[test]
    fn add_component_for_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();

        facade.add::<u32>(0, 1, 10);

        let components = facade.components.export();
        assert_eq!(components.len(), 1);
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10]));
        assert_eq!(facade.archetype_positions.get(0, 1), Some(0));
    }

    #[test]
    fn add_component_for_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 1, 10);

        facade.add::<u32>(0, 1, 20);

        let components = facade.components.export();
        assert_eq!(components.len(), 1);
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10, &20]));
        assert_eq!(facade.archetype_positions.get(0, 1), Some(0));
    }

    #[test]
    fn delete_archetype_for_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();

        facade.delete_archetype(0, 1);

        let components = facade.components.export();
        assert_eq!(components.len(), 1);
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), None);
    }

    #[test]
    fn delete_archetype_for_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 2, 10);
        facade.add::<u32>(0, 3, 20);
        facade.add::<u32>(0, 3, 30);

        facade.delete_archetype(0, 3);

        let components = facade.components.export();
        assert_eq!(components.len(), 1);
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10]));
        assert_option_iter!(type_components.iter::<u32>(1), Some(vec![]));
        assert_eq!(facade.archetype_positions.get(0, 2), Some(0));
        assert_eq!(facade.archetype_positions.get(0, 3), None);
    }

    #[test]
    fn retrieve_components() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 2, 10);
        facade.add::<u32>(0, 3, 20);
        facade.add::<u32>(0, 3, 30);

        let components = facade.components();

        let guard = components.read::<u32>().unwrap();
        assert_option_iter!(components.iter::<u32>(&guard, 2), Some(vec![&10]));
        assert_option_iter!(components.iter::<u32>(&guard, 3), Some(vec![&20, &30]));
    }

    #[test]
    fn retrieve_whether_component_exists_using_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();

        let exists = facade.exists::<u32>(0, EntityLocation::new(1, 2));

        assert!(!exists);
    }

    #[test]
    fn retrieve_whether_nonexisting_component_exists_using_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 1, 10);

        let exists = facade.exists::<u32>(0, EntityLocation::new(1, 2));

        assert!(!exists);
    }

    #[test]
    fn retrieve_whether_existing_component_exists_using_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 1, 10);
        facade.add::<u32>(0, 1, 20);
        facade.add::<u32>(0, 1, 30);

        let exists = facade.exists::<u32>(0, EntityLocation::new(1, 2));

        assert!(exists);
    }

    #[test]
    #[should_panic]
    fn replace_component_to_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();

        facade.replace::<u32>(0, EntityLocation::new(1, 2), 40);
    }

    #[test]
    fn replace_component_to_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 1, 10);
        facade.add::<u32>(0, 1, 20);
        facade.add::<u32>(0, 1, 30);

        facade.replace::<u32>(0, EntityLocation::new(1, 2), 40);

        let components = facade.components.export();
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10, &20, &40]));
    }

    #[test]
    #[should_panic]
    fn move_component_from_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();

        facade.move_(0, EntityLocation::new(2, 1), 4);
    }

    #[test]
    fn move_component_to_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 2, 10);
        facade.add::<u32>(0, 2, 20);
        facade.add::<u32>(0, 2, 30);
        facade.add::<u32>(0, 2, 40);

        facade.move_(0, EntityLocation::new(2, 1), 4);

        let components = facade.components.export();
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10, &40, &30]));
        assert_option_iter!(type_components.iter::<u32>(1), Some(vec![&20]));
        assert_eq!(facade.archetype_positions.get(0, 4), Some(1));
    }

    #[test]
    fn move_component_to_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 2, 10);
        facade.add::<u32>(0, 2, 20);
        facade.add::<u32>(0, 2, 30);
        facade.add::<u32>(0, 2, 40);
        facade.add::<u32>(0, 4, 50);

        facade.move_(0, EntityLocation::new(2, 1), 4);

        let components = facade.components.export();
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10, &40, &30]));
        assert_option_iter!(type_components.iter::<u32>(1), Some(vec![&50, &20]));
        assert_eq!(facade.archetype_positions.get(0, 4), Some(1));
    }

    #[test]
    #[should_panic]
    fn swap_remove_component_for_nonexisting_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();

        facade.swap_delete(0, EntityLocation::new(2, 1));
    }

    #[test]
    fn swap_delete_component_for_existing_archetype() {
        let mut facade = ComponentFacade::default();
        facade.create_type::<u32>();
        facade.add::<u32>(0, 2, 10);
        facade.add::<u32>(0, 2, 20);
        facade.add::<u32>(0, 2, 30);
        facade.add::<u32>(0, 2, 40);

        facade.swap_delete(0, EntityLocation::new(2, 1));

        let components = facade.components.export();
        let type_components = components[0].read().unwrap();
        assert_option_iter!(type_components.iter::<u32>(0), Some(vec![&10, &40, &30]));
    }
}
