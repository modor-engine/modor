use crate::internal::archetypes::data::MissingComponentError;
use crate::internal::archetypes::ArchetypeFacade;
use crate::internal::core::storages::{ComponentTypeStorage, EntityMainComponentTypeStorage};
use crate::internal::entities::data::EntityLocation;
use crate::internal::entities::EntityFacade;
use crate::internal::groups::GroupFacade;
use crate::ArchetypeInfo;
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;

mod storages;

#[derive(Default)]
pub(crate) struct CoreFacade {
    groups: GroupFacade,
    archetypes: ArchetypeFacade,
    entities: EntityFacade,
    component_types: ComponentTypeStorage,
    entity_main_component_types: EntityMainComponentTypeStorage,
}

impl CoreFacade {
    pub(super) fn group_archetype_idxs(
        &self,
        group_idx: NonZeroUsize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.archetypes.idxs_with_group(group_idx)
    }

    pub(super) fn group_component_type_idxs(
        &self,
        group_idx: NonZeroUsize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.archetypes.group_type_idxs(group_idx)
    }

    pub(super) fn create_group(&mut self) -> NonZeroUsize {
        self.groups.create()
    }

    pub(super) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        for entity_idx in self.groups.entity_idxs(group_idx) {
            self.entities.delete(entity_idx);
        }
        self.archetypes.delete_all(group_idx);
        self.groups.delete(group_idx);
    }

    pub(super) fn archetype_type_idxs(&self, archetype_idx: usize) -> &[usize] {
        self.archetypes.type_idxs(archetype_idx)
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
            .map(|&t| self.component_types.idx(t))
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

    pub(super) fn entity_location(&self, entity_idx: usize) -> Option<EntityLocation> {
        self.entities.location(entity_idx)
    }

    pub(super) fn add_entity_main_component_type<C>(&mut self) -> bool
    where
        C: Any,
    {
        self.entity_main_component_types.add(TypeId::of::<C>())
    }

    pub(super) fn create_entity(&mut self, group_idx: NonZeroUsize) -> usize {
        let entity_idx = self.entities.create();
        self.groups.add_entity(group_idx, entity_idx);
        entity_idx
    }

    pub(super) fn delete_entity(&mut self, entity_idx: usize) {
        self.entities.delete(entity_idx);
        self.groups.delete_entity(entity_idx);
    }

    pub(super) fn component_type_idx(&self, type_id: TypeId) -> Option<usize> {
        self.component_types.idx(type_id)
    }

    pub(super) fn add_component_type(&mut self, type_id: TypeId) -> usize {
        self.component_types.add(type_id)
    }

    pub(super) fn add_component(&mut self, entity_idx: usize, type_idx: usize) -> usize {
        let group_idx = self.groups.idx(entity_idx);
        let old_archetype_idx = self.entities.location(entity_idx).map(|a| a.archetype_idx);
        let new_archetype_idx =
            self.archetypes
                .add_component(group_idx, old_archetype_idx, type_idx);
        self.entities.move_(entity_idx, Some(new_archetype_idx));
        new_archetype_idx
    }

    pub(super) fn delete_component(
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
        assert!(!facade.entity_main_component_types.add(TypeId::of::<u32>()));
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
    fn add_component_type() {
        let mut facade = CoreFacade::default();
        let type_id = TypeId::of::<u32>();

        let type_idx = facade.add_component_type(type_id);

        assert_eq!(type_idx, 0);
        assert_eq!(facade.component_types.add(TypeId::of::<i64>()), 1);
    }

    #[test]
    fn add_component_for_entity_without_archetype() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component_type(TypeId::of::<i64>());
        facade.add_component_type(TypeId::of::<String>());
        let component_type_idx = facade.add_component_type(TypeId::of::<u32>());

        let archetype_idx = facade.add_component(entity_idx, component_type_idx);

        assert_eq!(archetype_idx, 0);
        assert_eq!(facade.archetypes.type_idxs(0), [component_type_idx]);
        assert_eq!(facade.archetypes.group_idx(0), group_idx);
        let location = EntityLocation::new(0, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
    }

    #[test]
    fn add_component_for_entity_with_archetype() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component_type(TypeId::of::<i64>());
        let component_type1_idx = facade.add_component_type(TypeId::of::<String>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<u32>());
        facade.add_component(entity_idx, component_type1_idx);

        let archetype_idx = facade.add_component(entity_idx, component_type2_idx);

        assert_eq!(archetype_idx, 1);
        let archetype_type_idxs = [component_type1_idx, component_type2_idx];
        assert_eq!(facade.archetypes.type_idxs(1), archetype_type_idxs);
        assert_eq!(facade.archetypes.group_idx(1), group_idx);
        let location = EntityLocation::new(1, 0);
        assert_eq!(facade.entities.location(entity_idx), Some(location));
    }

    #[test]
    fn retrieve_group_archetypes() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<i64>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<u32>());
        let archetype1_idx = facade.add_component(entity_idx, component_type1_idx);
        let archetype2_idx = facade.add_component(entity_idx, component_type2_idx);

        let archetype_idxs = facade.group_archetype_idxs(group_idx);

        assert_iter!(archetype_idxs, [archetype1_idx, archetype2_idx]);
    }

    #[test]
    fn retrieve_group_component_types() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<i64>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<u32>());
        facade.add_component(entity_idx, component_type1_idx);
        facade.add_component(entity_idx, component_type2_idx);

        let component_type_idxs = facade.group_component_type_idxs(group_idx);

        let group_component_type_idxs = [component_type1_idx, component_type2_idx];
        assert_iter!(component_type_idxs, group_component_type_idxs);
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
        let component_type1_idx = facade.add_component_type(TypeId::of::<i64>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<u32>());
        facade.add_component(entity1_idx, component_type1_idx);
        let archetype2_idx = facade.add_component(entity2_idx, component_type2_idx);

        facade.delete_group(group1_idx);

        assert_eq!(facade.archetypes.group_type_idxs(group1_idx).next(), None);
        let actual_group2_type_idxs = facade.archetypes.group_type_idxs(group2_idx);
        assert_iter!(actual_group2_type_idxs, [component_type2_idx]);
        assert_eq!(facade.entities.location(entity1_idx), None);
        let location = Some(EntityLocation::new(archetype2_idx, 0));
        assert_eq!(facade.entities.location(entity2_idx), location);
        assert_panics!(facade.groups.idx(entity1_idx));
        assert_eq!(facade.groups.idx(entity2_idx), group2_idx);
        assert_eq!(facade.groups.create(), group1_idx);
    }

    #[test]
    fn retrieve_archetype_types() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<i64>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<u32>());
        let archetype1_idx = facade.add_component(entity_idx, component_type1_idx);
        let archetype2_idx = facade.add_component(entity_idx, component_type2_idx);

        let type_idxs = facade.archetype_type_idxs(archetype2_idx);

        assert_eq!(type_idxs, [archetype1_idx, archetype2_idx]);
    }

    #[test]
    fn retrieve_archetype_entity_idxs() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group_idx);
        let entity2_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<u32>());
        let archetype_idx = facade.add_component(entity1_idx, component_type1_idx);
        facade.add_component(entity2_idx, component_type1_idx);

        let entity_idxs = facade.archetype_entity_idxs(archetype_idx);

        assert_eq!(entity_idxs, [entity1_idx, entity2_idx]);
    }

    #[test]
    fn retrieve_archetype_from_too_much_component_types_and_group() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        let component_type_idx = facade.add_component_type(type1_id);
        facade.add_component(entity_idx, component_type_idx);

        let archetypes = facade.archetypes(&[type1_id, type2_id], Some(group_idx));

        assert_eq!(archetypes, []);
    }

    #[test]
    fn retrieve_archetype_from_existing_component_types_and_group() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        let component_type1_idx = facade.add_component_type(type1_id);
        let component_type2_idx = facade.add_component_type(type2_id);
        facade.add_component(entity_idx, component_type1_idx);
        let archetype2_idx = facade.add_component(entity_idx, component_type2_idx);

        let archetypes = facade.archetypes(&[type2_id], Some(group_idx));

        assert_eq!(archetypes, [ArchetypeInfo::new(archetype2_idx, group_idx)]);
    }

    #[test]
    fn retrieve_archetype_from_existing_component_types_and_no_group() {
        let mut facade = CoreFacade::default();
        let group1_idx = facade.create_group();
        let group2_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group1_idx);
        let entity2_idx = facade.create_entity(group2_idx);
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        let component_type1_idx = facade.add_component_type(type1_id);
        let component_type2_idx = facade.add_component_type(type2_id);
        facade.add_component(entity1_idx, component_type1_idx);
        let archetype2_idx = facade.add_component(entity1_idx, component_type2_idx);
        let archetype3_idx = facade.add_component(entity2_idx, component_type2_idx);

        let archetypes = facade.archetypes(&[type2_id], None);

        let archetype2_info = ArchetypeInfo::new(archetype2_idx, group1_idx);
        let archetype3_info = ArchetypeInfo::new(archetype3_idx, group2_idx);
        assert_eq!(archetypes, [archetype2_info, archetype3_info]);
    }

    #[test]
    fn retrieve_entity_location() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group_idx);
        let entity2_idx = facade.create_entity(group_idx);
        let component_type_idx = facade.add_component_type(TypeId::of::<u32>());
        let archetype_idx = facade.add_component(entity1_idx, component_type_idx);
        facade.add_component(entity2_idx, component_type_idx);

        let actual_location = facade.entity_location(entity2_idx);

        assert_eq!(actual_location, Some(EntityLocation::new(archetype_idx, 1)));
    }

    #[test]
    fn delete_entity() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group_idx);
        let entity2_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<i64>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<u32>());
        facade.add_component(entity1_idx, component_type1_idx);
        let archetype_idx = facade.add_component(entity2_idx, component_type2_idx);

        facade.delete_entity(entity1_idx);

        assert_eq!(facade.entities.location(entity1_idx), None);
        let location = EntityLocation::new(archetype_idx, 0);
        assert_eq!(facade.entities.location(entity2_idx), Some(location));
        assert_panics!(facade.groups.idx(entity1_idx));
        assert_eq!(facade.groups.idx(entity2_idx), group_idx);
    }

    #[test]
    fn retrieve_component_type_idx_from_type_id() {
        let mut facade = CoreFacade::default();
        let type1_id = TypeId::of::<u32>();
        let type2_id = TypeId::of::<i64>();
        facade.add_component_type(type1_id);
        let component_type_idx = facade.add_component_type(type2_id);

        let type_idx = facade.component_type_idx(type2_id);

        assert_eq!(type_idx, Some(component_type_idx));
    }

    #[test]
    fn delete_existing_component() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<u32>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<i64>());
        facade.add_component(entity_idx, component_type1_idx);
        let last_archetype_idx = facade.add_component(entity_idx, component_type2_idx);

        let archetype_idx = facade.delete_component(entity_idx, component_type1_idx);

        let expected_archetype_idx = last_archetype_idx + 1;
        assert_eq!(archetype_idx, Ok(Some(expected_archetype_idx)));
        let location = Some(EntityLocation::new(expected_archetype_idx, 0));
        assert_eq!(facade.entities.location(entity_idx), location);
        let actual_group_idx = facade.archetypes.group_idx(expected_archetype_idx);
        assert_eq!(actual_group_idx, group_idx);
        let actual_type_idxs = facade.archetypes.type_idxs(expected_archetype_idx);
        assert_eq!(actual_type_idxs, [component_type2_idx]);
    }

    #[test]
    fn delete_missing_component_from_empty_entity() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let component_type_idx = facade.add_component_type(TypeId::of::<u32>());

        let archetype_idx = facade.delete_component(entity_idx, component_type_idx);

        assert_eq!(archetype_idx, Err(MissingComponentError));
    }

    #[test]
    fn delete_missing_component_from_nonempty_entity() {
        let mut facade = CoreFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let component_type1_idx = facade.add_component_type(TypeId::of::<u32>());
        let component_type2_idx = facade.add_component_type(TypeId::of::<i64>());
        facade.add_component(entity_idx, component_type1_idx);

        let archetype_idx = facade.delete_component(entity_idx, component_type2_idx);

        assert_eq!(archetype_idx, Err(MissingComponentError));
    }
}
