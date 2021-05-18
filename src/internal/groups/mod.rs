use self::storages::{EntityGroupStorage, EntityStorage, GroupStorage};
use std::num::NonZeroUsize;

mod storages;

#[derive(Default)]
pub(super) struct GroupFacade {
    groups: GroupStorage,
    entities: EntityStorage,
    entity_groups: EntityGroupStorage,
}

impl GroupFacade {
    pub(super) fn idx(&mut self, entity_idx: usize) -> NonZeroUsize {
        self.entity_groups.idx(entity_idx)
    }

    pub(super) fn entity_idxs(
        &mut self,
        group_idx: NonZeroUsize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.entities.idxs(group_idx)
    }

    pub(super) fn create(&mut self) -> NonZeroUsize {
        self.groups.create()
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        for entity_idx in self.entities.idxs(group_idx) {
            self.entity_groups.delete(entity_idx);
        }
        self.groups.delete(group_idx);
        self.entities.delete_group(group_idx);
    }

    pub(super) fn add_entity(&mut self, group_idx: NonZeroUsize, entity_idx: usize) {
        self.entities.add(entity_idx, group_idx);
        self.entity_groups.set(entity_idx, group_idx);
    }

    pub(super) fn delete_entity(&mut self, entity_idx: usize) {
        let group_idx = self.entity_groups.idx(entity_idx);
        self.entities.delete(entity_idx, group_idx);
        self.entity_groups.delete(entity_idx);
    }
}

#[cfg(test)]
mod tests_group_facade {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn create_group() {
        let mut facade = GroupFacade::default();

        let group_idx = facade.create();

        assert_eq!(group_idx, 1.try_into().unwrap());
        assert_eq!(facade.groups.create(), 2.try_into().unwrap());
    }

    #[test]
    fn add_entity_to_group() {
        let mut facade = GroupFacade::default();

        facade.add_entity(2.try_into().unwrap(), 3);

        assert_iter!(facade.entities.idxs(2.try_into().unwrap()), [3]);
        assert_eq!(facade.entity_groups.idx(3), 2.try_into().unwrap());
    }

    #[test]
    fn retrieve_idx() {
        let mut facade = GroupFacade::default();
        let group_idx = facade.create();
        facade.add_entity(group_idx, 2);

        let group_idx = facade.idx(2);

        assert_eq!(group_idx, 1.try_into().unwrap());
    }

    #[test]
    fn retrieve_enity_idxs() {
        let mut facade = GroupFacade::default();
        let group_idx = facade.create();
        facade.add_entity(group_idx, 2);
        facade.add_entity(group_idx, 3);

        let entity_idxs = facade.entity_idxs(group_idx);

        assert_iter!(entity_idxs, [2, 3]);
    }

    #[test]
    fn delete_group() {
        let mut facade = GroupFacade::default();
        let group1_idx = facade.create();
        let group2_idx = facade.create();
        facade.add_entity(group1_idx, 3);
        facade.add_entity(group1_idx, 4);
        facade.add_entity(group2_idx, 5);

        facade.delete(group1_idx);

        assert_eq!(facade.entities.idxs(group1_idx).next(), None);
        assert_iter!(facade.entities.idxs(group2_idx), [5]);
        assert_panics!(facade.entity_groups.idx(3));
        assert_panics!(facade.entity_groups.idx(4));
        assert_eq!(facade.entity_groups.idx(5), group2_idx);
        assert_eq!(facade.groups.create(), group1_idx);
    }

    #[test]
    fn delete_entity() {
        let mut facade = GroupFacade::default();
        let group_idx = facade.create();
        facade.add_entity(group_idx, 3);
        facade.add_entity(group_idx, 4);

        facade.delete_entity(3);

        assert_iter!(facade.entities.idxs(group_idx), [4]);
        assert_panics!(facade.entity_groups.idx(3));
        assert_eq!(facade.entity_groups.idx(4), group_idx);
    }
}
