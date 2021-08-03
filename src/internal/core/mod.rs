use crate::external::systems::definition::internal::ArchetypeInfo;
use crate::internal::archetypes::data::{ExistingComponentError, MissingComponentError};
use crate::internal::archetypes::ArchetypeFacade;
use crate::internal::components::data::{ComponentReadGuard, ComponentWriteGuard};
use crate::internal::components::ComponentFacade;
use crate::internal::entities::EntityFacade;
use crate::internal::groups::GroupFacade;
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;

#[derive(Default)]
pub(crate) struct CoreFacade {
    groups: GroupFacade,
    archetypes: ArchetypeFacade,
    entities: EntityFacade,
    components: ComponentFacade,
}

impl CoreFacade {
    pub(super) fn create_group(&mut self) -> NonZeroUsize {
        self.groups.create()
    }

    pub(super) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        for type_idxs in self.archetypes.group_type_idxs(group_idx) {
            for archetype_idx in self.archetypes.idxs_with_group(group_idx) {
                self.components.delete_archetype(type_idxs, archetype_idx);
            }
        }
        for entity_idx in self.groups.entity_idxs(group_idx) {
            self.entities.delete(entity_idx);
        }
        self.archetypes.delete_all(group_idx);
        self.groups.delete(group_idx);
    }

    pub(crate) fn archetype_entity_idxs(&self, archetype_idx: usize) -> &[usize] {
        self.entities.idxs(archetype_idx)
    }

    pub(crate) fn archetypes(
        &self,
        component_types: &[TypeId],
        group_idx: Option<NonZeroUsize>,
    ) -> Vec<ArchetypeInfo> {
        let type_idxs: Vec<_> = component_types
            .iter()
            .map(|&t| self.components.type_idx(t))
            .collect();
        if type_idxs.iter().any(|&t| t == None) {
            return Vec::new();
        }
        let type_idxs: Vec<_> = type_idxs.into_iter().flatten().collect();
        self.archetypes
            .idxs_with_types(&type_idxs)
            .into_iter()
            .filter(|&a| group_idx.map_or(true, |g| self.archetypes.group_idx(a) == g))
            .map(|a| ArchetypeInfo::new(a, self.archetypes.group_idx(a)))
            .collect()
    }

    pub(super) fn add_entity_main_component_type<C>(&mut self) -> bool
    where
        C: Any,
    {
        self.components
            .add_entity_main_component_type(TypeId::of::<C>())
    }

    pub(super) fn create_entity(&mut self, group_idx: NonZeroUsize) -> usize {
        let entity_idx = self.entities.create();
        self.groups.add_entity(group_idx, entity_idx);
        entity_idx
    }

    pub(super) fn delete_entity(&mut self, entity_idx: usize) {
        if let Some(location) = self.entities.location(entity_idx) {
            let component_type_idxs = self.archetypes.type_idxs(location.archetype_idx);
            self.components.delete_entity(component_type_idxs, location);
        }
        self.entities.delete(entity_idx);
        self.groups.delete_entity(entity_idx);
    }

    pub(crate) fn read_components<C>(&self) -> Option<ComponentReadGuard<'_, C>>
    where
        C: Any,
    {
        self.components.read_components::<C>()
    }

    pub(crate) fn write_components<C>(&self) -> Option<ComponentWriteGuard<'_, C>>
    where
        C: Any,
    {
        self.components.write_components::<C>()
    }

    pub(super) fn add_component<C>(&mut self, entity_idx: usize, component: C)
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.components.type_idx_or_create::<C>();
        let location = self.entities.location(entity_idx);
        if let Ok(new_archetype_idx) = self.add_component_type_to_entity(entity_idx, type_idx) {
            let archetypes = &self.archetypes;
            let moved_type_idxs: &[usize] =
                location.map_or(&[], |l| archetypes.type_idxs(l.archetype_idx));
            self.components.add(
                moved_type_idxs,
                location,
                new_archetype_idx,
                type_idx,
                component,
            );
        } else {
            let location = location
                .expect("internal error: entity location not found but existing component type");
            self.components.replace(type_idx, location, component);
        }
    }

    pub(super) fn delete_component(&mut self, entity_idx: usize, component_type: TypeId) {
        self.delete_entity_internal(entity_idx, component_type);
    }

    fn add_component_type_to_entity(
        &mut self,
        entity_idx: usize,
        type_idx: usize,
    ) -> Result<usize, ExistingComponentError> {
        let group_idx = self.groups.idx(entity_idx);
        let old_archetype_idx = self.entities.location(entity_idx).map(|a| a.archetype_idx);
        let new_archetype_idx =
            self.archetypes
                .add_component(group_idx, old_archetype_idx, type_idx)?;
        self.entities.move_(entity_idx, Some(new_archetype_idx));
        Ok(new_archetype_idx)
    }

    fn delete_entity_internal(&mut self, entity_idx: usize, component_type: TypeId) -> Option<()> {
        let type_idx = self.components.type_idx(component_type)?;
        let location = self.entities.location(entity_idx)?;
        let new_archetype_idx = self
            .delete_component_type_from_entity(entity_idx, type_idx)
            .ok()?;
        let moved_type_idx = self.archetypes.type_idxs(location.archetype_idx);
        self.components
            .delete(moved_type_idx, location, new_archetype_idx, type_idx);
        Some(())
    }

    fn delete_component_type_from_entity(
        &mut self,
        entity_idx: usize,
        type_idx: usize,
    ) -> Result<Option<usize>, MissingComponentError> {
        let src_archetype_idx = self
            .entities
            .location(entity_idx)
            .ok_or(MissingComponentError)?
            .archetype_idx;
        let dst_archetype_idx = self
            .archetypes
            .delete_component(src_archetype_idx, type_idx)?;
        self.entities.move_(entity_idx, dst_archetype_idx);
        Ok(dst_archetype_idx)
    }
}

#[cfg(test)]
mod core_facade_tests {
    use super::*;
    use crate::internal::entities::data::EntityLocation;
    use std::convert::TryInto;

    #[test]
    fn create_group() {
        let mut facade = CoreFacade::default();

        let actual_group_idx = facade.create_group();

        assert_eq!(actual_group_idx, 1.try_into().unwrap());
        assert_eq!(facade.groups.create(), 2.try_into().unwrap());
    }

    #[test]
    fn add_entity_main_component_type() {
        let mut facade = CoreFacade::default();

        let new_type = facade.add_entity_main_component_type::<u32>();

        assert!(new_type);
        let type_id = TypeId::of::<u32>();
        assert!(!facade.components.add_entity_main_component_type(type_id));
    }

    #[test]
    fn create_entity() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();

        let entity_idx = facade.create_entity(group_idx);

        assert_eq!(entity_idx, 0);
        assert_eq!(facade.groups.idx(entity_idx), group_idx);
        assert_eq!(facade.entities.create(), 1);
    }

    #[test]
    fn add_component_for_entity_without_component() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);

        facade.add_component(entity_idx, 10_u32);

        assert_eq!(facade.archetypes.type_idxs(0), &[0]);
        let location = EntityLocation::new(0, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
        let components = facade.components.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![&10]));
    }

    #[test]
    fn add_component_for_entity_with_components() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);

        facade.add_component(entity_idx, 20_i64);

        assert_eq!(facade.archetypes.type_idxs(0), &[0]);
        assert_eq!(facade.archetypes.type_idxs(1), &[0, 1]);
        let location = EntityLocation::new(1, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
        let components = facade.components.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(1), Some(vec![&10]));
        let components = facade.components.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(1), Some(vec![&20]));
    }

    #[test]
    fn add_component_for_entity_with_same_component_type() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);

        facade.add_component(entity_idx, 20_u32);

        assert_eq!(facade.archetypes.type_idxs(0), &[0]);
        let location = EntityLocation::new(0, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
        let components = facade.components.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![&20]));
    }

    #[test]
    fn delete_group() {
        let mut facade = CoreFacade::default();
        let group1_idx = facade.create_group();
        let group2_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group1_idx);
        let entity2_idx = facade.create_entity(group2_idx);
        facade.add_entity_main_component_type::<i64>();
        facade.add_entity_main_component_type::<u32>();
        facade.add_component(entity1_idx, 10_u32);
        facade.add_component(entity2_idx, 20_i64);

        facade.delete_group(group1_idx);

        assert_eq!(facade.archetypes.group_type_idxs(group1_idx).next(), None);
        let actual_group2_type_idxs = facade.archetypes.group_type_idxs(group2_idx);
        assert_iter!(actual_group2_type_idxs, [1]);
        assert_eq!(facade.entities.location(entity1_idx), None);
        let location = Some(EntityLocation::new(1, 0));
        assert_eq!(facade.entities.location(entity2_idx), location);
        assert_panics!(facade.groups.idx(entity1_idx));
        assert_eq!(facade.groups.idx(entity2_idx), group2_idx);
        assert_eq!(facade.groups.create(), group1_idx);
        let components = facade.components.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        let components = facade.components.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(1), Some(vec![&20]));
    }

    #[test]
    fn retrieve_archetype_entity_idxs() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);

        let entity_idxs = facade.archetype_entity_idxs(0);

        assert_eq!(entity_idxs, &[entity_idx]);
    }

    #[test]
    fn retrieve_archetype_from_too_much_component_types_and_group() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_usize);
        let type_ids = &[TypeId::of::<u32>(), TypeId::of::<i64>()];

        let archetypes = facade.archetypes(type_ids, Some(group_idx));

        assert_eq!(archetypes, []);
    }

    #[test]
    fn retrieve_archetype_from_existing_component_types_and_group() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);
        facade.add_component(entity_idx, 20_i64);
        let type_ids = &[TypeId::of::<i64>()];

        let archetypes = facade.archetypes(type_ids, Some(group_idx));

        assert_eq!(archetypes, [ArchetypeInfo::new(1, group_idx)]);
    }

    #[test]
    fn retrieve_archetype_from_existing_component_types_and_no_group() {
        let mut facade = CoreFacade::default();
        let group1_idx = facade.create_group();
        let group2_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group1_idx);
        let entity2_idx = facade.create_entity(group2_idx);
        facade.add_component(entity1_idx, 10_u32);
        facade.add_component(entity1_idx, 20_i64);
        facade.add_component(entity2_idx, 30_i64);
        let type_ids = &[TypeId::of::<i64>()];

        let archetypes = facade.archetypes(type_ids, None);

        let archetype2_info = ArchetypeInfo::new(1, group1_idx);
        let archetype3_info = ArchetypeInfo::new(2, group2_idx);
        assert_eq!(archetypes, [archetype2_info, archetype3_info]);
    }

    #[test]
    fn delete_entity() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group_idx);
        let entity2_idx = facade.create_entity(group_idx);
        facade.add_component(entity1_idx, 10_u32);
        facade.add_component(entity2_idx, 20_i64);

        facade.delete_entity(entity1_idx);

        assert_eq!(facade.entities.location(entity1_idx), None);
        let location = EntityLocation::new(1, 0);
        assert_eq!(facade.entities.location(entity2_idx), Some(location));
        assert_panics!(facade.groups.idx(entity1_idx));
        assert_eq!(facade.groups.idx(entity2_idx), group_idx);
    }

    #[test]
    fn write_components() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);

        let mut components = facade.write_components::<u32>().unwrap();

        assert_option_iter!(components.archetype_iter_mut(0), Some(vec![&mut 10]));
        assert_option_iter!(components.archetype_iter_mut(1), None);
    }

    #[test]
    fn delete_component_with_missing_type() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);

        facade.delete_component(entity_idx, TypeId::of::<u32>());

        assert_eq!(facade.entities.location(entity_idx), None);
        assert!(facade.components.read_components::<u32>().is_none());
    }

    #[test]
    fn delete_existing_component() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);
        facade.add_component(entity_idx, 20_i64);

        facade.delete_component(entity_idx, TypeId::of::<u32>());

        let location = EntityLocation::new(2, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
        let components = facade.components.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        assert_option_iter!(components.archetype_iter(2), None);
        let components = facade.components.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(2), Some(vec![&20]));
    }

    #[test]
    fn delete_not_existing_component() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component(entity_idx, 10_u32);
        facade.add_component(entity_idx, 20_i64);
        facade.delete_component(entity_idx, TypeId::of::<u32>());

        facade.delete_component(entity_idx, TypeId::of::<u32>());

        let location = EntityLocation::new(2, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
        let components = facade.components.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        assert_option_iter!(components.archetype_iter(2), None);
        let components = facade.components.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(2), Some(vec![&20]));
    }
}
