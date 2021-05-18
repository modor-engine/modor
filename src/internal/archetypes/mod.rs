use crate::internal::archetypes::data::MissingComponentError;
use crate::internal::archetypes::storages::{
    GroupArchetypeStorage, NextArchetypeStorage, PreviousArchetypeStorage, PropertyStorage,
    TypeArchetypeStorage,
};
use itertools::Itertools;
use std::num::NonZeroUsize;

pub(super) mod data;
mod storages;

#[derive(Default)]
pub(super) struct ArchetypeFacade {
    properties: PropertyStorage,
    group_archetypes: GroupArchetypeStorage,
    type_archetypes: TypeArchetypeStorage,
    next_archetypes: NextArchetypeStorage,
    previous_archetypes: PreviousArchetypeStorage,
}

impl ArchetypeFacade {
    pub(super) fn group_type_idxs(
        &self,
        group_idx: NonZeroUsize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.group_archetypes
            .idxs(group_idx)
            .flat_map(move |a| self.properties.type_idxs(a).iter().copied())
            .unique()
    }

    pub(super) fn group_idx(&self, archetype_idx: usize) -> NonZeroUsize {
        self.properties.group_idx(archetype_idx)
    }

    pub(super) fn type_idxs(&self, archetype_idx: usize) -> &[usize] {
        self.properties.type_idxs(archetype_idx)
    }

    pub(super) fn idxs_with_types(&self, type_idxs: &[usize]) -> Vec<usize> {
        self.type_archetypes.idxs(type_idxs)
    }

    pub(super) fn idxs_with_group(
        &self,
        group_idx: NonZeroUsize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.group_archetypes.idxs(group_idx)
    }

    pub(super) fn delete_all(&mut self, group_idx: NonZeroUsize) {
        self.next_archetypes.delete(group_idx);
        self.previous_archetypes.delete(group_idx);
        for archetype_idx in self.group_archetypes.remove(group_idx) {
            for &type_idx in self.properties.type_idxs(archetype_idx) {
                self.type_archetypes.delete(type_idx, archetype_idx);
            }
            self.properties.delete(archetype_idx);
        }
    }

    #[allow(clippy::option_if_let_else)]
    pub(super) fn add_component(
        &mut self,
        group_idx: NonZeroUsize,
        src_archetype_idx: Option<usize>,
        type_idx: usize,
    ) -> usize {
        if let Some(dst_archetype_idx) =
            self.next_archetypes
                .idx(group_idx, src_archetype_idx, type_idx)
        {
            dst_archetype_idx
        } else if let Some(dst_archetype_idx) =
            self.properties
                .next_idx(group_idx, src_archetype_idx, type_idx)
        {
            let next_archetypes = &mut self.next_archetypes;
            next_archetypes.add(group_idx, src_archetype_idx, type_idx, dst_archetype_idx);
            dst_archetype_idx
        } else {
            self.create_next_archetype(src_archetype_idx, type_idx, group_idx)
        }
    }

    #[allow(clippy::option_if_let_else)]
    pub(super) fn delete_component(
        &mut self,
        src_archetype_idx: usize,
        type_idx: usize,
    ) -> Result<Option<usize>, MissingComponentError> {
        let group_idx = self.properties.group_idx(src_archetype_idx);
        Ok(
            if let Some(dst_archetype_idx) =
                self.previous_archetypes
                    .idx(group_idx, src_archetype_idx, type_idx)
            {
                dst_archetype_idx
            } else if let Some(dst_archetype_idx) =
                self.properties
                    .previous_idx(group_idx, src_archetype_idx, type_idx)?
            {
                let previous_archetypes = &mut self.previous_archetypes;
                previous_archetypes.add(group_idx, src_archetype_idx, type_idx, dst_archetype_idx);
                dst_archetype_idx
            } else {
                self.create_previous_archetype(src_archetype_idx, type_idx, group_idx)
            },
        )
    }

    fn create_next_archetype(
        &mut self,
        src_archetype_idx: Option<usize>,
        new_type_idx: usize,
        group_idx: NonZeroUsize,
    ) -> usize {
        let properties = &mut self.properties;
        let dst_archetype_idx = properties.create_next(group_idx, src_archetype_idx, new_type_idx);
        self.group_archetypes.add(group_idx, dst_archetype_idx);
        for &type_idx in properties.type_idxs(dst_archetype_idx) {
            self.type_archetypes.add(type_idx, dst_archetype_idx);
        }
        self.next_archetypes.add(
            group_idx,
            src_archetype_idx,
            new_type_idx,
            dst_archetype_idx,
        );
        dst_archetype_idx
    }

    fn create_previous_archetype(
        &mut self,
        src_archetype_idx: usize,
        deleted_type_idx: usize,
        group_idx: NonZeroUsize,
    ) -> Option<usize> {
        let properties = &mut self.properties;
        let dst_archetype_idx =
            properties.create_previous(group_idx, src_archetype_idx, deleted_type_idx);
        if let Some(archetype_idx) = dst_archetype_idx {
            self.group_archetypes.add(group_idx, archetype_idx);
            for &type_idx in properties.type_idxs(archetype_idx) {
                self.type_archetypes.add(type_idx, archetype_idx);
            }
        }
        self.previous_archetypes.add(
            group_idx,
            src_archetype_idx,
            deleted_type_idx,
            dst_archetype_idx,
        );
        dst_archetype_idx
    }
}

#[cfg(test)]
mod tests_archetype_facade {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn add_component_in_first_new_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();

        let archetype_idx = facade.add_component(group_idx, None, 2);

        assert_eq!(archetype_idx, 0);
        assert_eq!(facade.properties.type_idxs(0), [2]);
        assert_iter!(facade.group_archetypes.idxs(group_idx), [0]);
        assert_eq!(facade.type_archetypes.idxs(&[2]), [0]);
        assert_eq!(facade.next_archetypes.idx(group_idx, None, 2), Some(0));
    }

    #[test]
    fn add_component_in_other_new_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);

        let archetype_idx = facade.add_component(group_idx, None, 3);

        assert_eq!(archetype_idx, 1);
        assert_eq!(facade.properties.type_idxs(1), [3]);
        assert_iter!(facade.group_archetypes.idxs(group_idx), [0, 1]);
        assert_eq!(facade.type_archetypes.idxs(&[3]), [1]);
        assert_eq!(facade.next_archetypes.idx(group_idx, None, 3), Some(1));
    }

    #[test]
    fn add_component_in_existing_and_directly_accessible_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 3);

        let archetype_idx = facade.add_component(group_idx, None, 3);

        assert_eq!(archetype_idx, 0);
    }

    #[test]
    fn add_component_in_existing_and_not_directly_accessible_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 4);
        facade.add_component(group_idx, None, 5);
        facade.add_component(group_idx, Some(0), 5);

        let archetype_idx = facade.add_component(group_idx, Some(1), 4);

        assert_eq!(archetype_idx, 2);
        assert_eq!(facade.next_archetypes.idx(group_idx, Some(1), 4), Some(2));
    }

    #[test]
    fn delete_component_to_existing_and_not_directly_accessible_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);
        facade.add_component(group_idx, Some(0), 3);

        let archetype_idx = facade.delete_component(1, 3);

        assert_eq!(archetype_idx, Ok(Some(0)));
        assert_eq!(
            facade.previous_archetypes.idx(group_idx, 1, 3),
            Some(Some(0))
        );
    }

    #[test]
    fn delete_component_to_existing_and_directly_accessible_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);
        facade.add_component(group_idx, Some(0), 3);
        let _ = facade.delete_component(1, 3);

        let archetype_idx = facade.delete_component(1, 3);

        assert_eq!(archetype_idx, Ok(Some(0)));
        assert_eq!(
            facade.previous_archetypes.idx(group_idx, 1, 3),
            Some(Some(0))
        );
    }

    #[test]
    fn delete_missing_component_from_existing_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);
        facade.add_component(group_idx, Some(0), 3);

        let archetype_idx = facade.delete_component(1, 4);

        assert_eq!(archetype_idx, Err(MissingComponentError));
        assert_eq!(facade.previous_archetypes.idx(group_idx, 1, 4), None);
    }

    #[test]
    fn delete_component_to_new_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);
        facade.add_component(group_idx, Some(0), 3);

        let archetype_idx = facade.delete_component(1, 2);

        assert_eq!(archetype_idx, Ok(Some(2)));
        assert_eq!(facade.properties.type_idxs(2), [3]);
        assert_eq!(facade.properties.group_idx(2), group_idx);
        assert_iter!(facade.group_archetypes.idxs(group_idx), [0, 1, 2]);
        assert_eq!(facade.type_archetypes.idxs(&[3]), [1, 2]);
        assert_eq!(
            facade.previous_archetypes.idx(group_idx, 1, 2),
            Some(Some(2))
        );
    }

    #[test]
    fn delete_component_to_no_archetype() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);

        let archetype_idx = facade.delete_component(0, 2);

        assert_eq!(archetype_idx, Ok(None));
        assert_eq!(facade.previous_archetypes.idx(group_idx, 0, 2), Some(None));
    }

    #[test]
    fn retrieve_group_component_types() {
        let mut facade = ArchetypeFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.add_component(group1_idx, None, 3);
        facade.add_component(group1_idx, None, 4);
        facade.add_component(group2_idx, Some(0), 5);

        let type_idxs = facade.group_type_idxs(group1_idx);

        assert_iter!(type_idxs, [3, 4]);
    }

    #[test]
    fn retrieve_archetype_group_idx() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);

        let actual_group_idx = facade.group_idx(0);

        assert_eq!(actual_group_idx, group_idx);
    }

    #[test]
    fn retrieve_archetype_type_idxs() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 2);
        facade.add_component(group_idx, Some(0), 3);

        let type_idxs = facade.type_idxs(1);

        assert_eq!(type_idxs, [2, 3]);
    }

    #[test]
    fn retrieve_archetypes_with_types() {
        let mut facade = ArchetypeFacade::default();
        let group_idx = 1.try_into().unwrap();
        facade.add_component(group_idx, None, 3);
        facade.add_component(group_idx, Some(0), 4);
        facade.add_component(group_idx, None, 4);

        let archetype_idxs = facade.idxs_with_types(&[4]);

        assert_eq!(archetype_idxs, vec![1, 2]);
    }

    #[test]
    fn retrieve_archetypes_with_group() {
        let mut facade = ArchetypeFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.add_component(group1_idx, None, 3);
        facade.add_component(group1_idx, Some(0), 4);
        facade.add_component(group2_idx, None, 4);

        let archetype_idxs = facade.idxs_with_group(group1_idx);

        assert_iter!(archetype_idxs, [0, 1]);
    }

    #[test]
    fn delete_all_archetypes_from_group() {
        let mut facade = ArchetypeFacade::default();
        let group1_idx = 1.try_into().unwrap();
        let group2_idx = 2.try_into().unwrap();
        facade.add_component(group1_idx, None, 3);
        facade.add_component(group1_idx, Some(0), 4);
        facade.add_component(group2_idx, None, 3);
        facade.add_component(group2_idx, Some(2), 6);
        let _ = facade.delete_component(2, 3);
        let _ = facade.delete_component(1, 4);
        let _ = facade.delete_component(0, 3);
        let _ = facade.delete_component(3, 6);

        facade.delete_all(group1_idx);

        let next_archetypes = &facade.next_archetypes;
        let previous_archetypes = &facade.previous_archetypes;
        assert_panics!(facade.properties.group_idx(0));
        assert_panics!(facade.properties.group_idx(1));
        assert_eq!(facade.properties.group_idx(2), group2_idx);
        assert_panics!(facade.properties.type_idxs(0));
        assert_panics!(facade.properties.type_idxs(1));
        assert_eq!(facade.properties.type_idxs(2), [3]);
        assert_eq!(facade.group_archetypes.idxs(group1_idx).next(), None);
        assert_iter!(facade.group_archetypes.idxs(group2_idx), [2, 3]);
        assert_eq!(facade.type_archetypes.idxs(&[3]), [3, 2]);
        assert_eq!(facade.type_archetypes.idxs(&[6]), [3]);
        assert_eq!(next_archetypes.idx(group1_idx, None, 3), None);
        assert_eq!(next_archetypes.idx(group1_idx, Some(0), 4), None);
        assert_eq!(next_archetypes.idx(group2_idx, None, 3), Some(2));
        assert_eq!(next_archetypes.idx(group2_idx, Some(2), 6), Some(3));
        assert_eq!(previous_archetypes.idx(group1_idx, 2, 3), None);
        assert_eq!(previous_archetypes.idx(group1_idx, 1, 4), None);
        assert_eq!(previous_archetypes.idx(group2_idx, 0, 3), None);
        assert_eq!(previous_archetypes.idx(group2_idx, 3, 6), Some(Some(2)));
    }
}
