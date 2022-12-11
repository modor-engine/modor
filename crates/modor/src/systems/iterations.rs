use super::context::Storages;
use crate::storages::archetypes::ArchetypeIdx;
use crate::storages::systems::SystemIdx;
use crate::ArchetypeFilterFn;
use std::slice::Iter;

#[derive(Clone)]
pub(crate) struct FilteredArchetypeIdxIter<'a> {
    pub(crate) archetype_idxs: Iter<'a, ArchetypeIdx>,
    pub(crate) archetype_filter_fn: ArchetypeFilterFn,
    pub(crate) storages: Storages<'a>,
    pub(crate) system_idx: Option<SystemIdx>,
}

impl Iterator for FilteredArchetypeIdxIter<'_> {
    type Item = ArchetypeIdx;

    fn next(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            &mut self.archetype_idxs,
            self.archetype_filter_fn,
            self.storages,
            self.system_idx,
        )
    }
}

impl DoubleEndedIterator for FilteredArchetypeIdxIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            (&mut self.archetype_idxs).rev(),
            self.archetype_filter_fn,
            self.storages,
            self.system_idx,
        )
    }
}

impl FilteredArchetypeIdxIter<'_> {
    fn next_idx<'a, I>(
        mut archetype_idxs: I,
        is_archetype_kept_fn: ArchetypeFilterFn,
        storages: Storages<'a>,
        system_idx: Option<SystemIdx>,
    ) -> Option<ArchetypeIdx>
    where
        I: Iterator<Item = &'a ArchetypeIdx>,
    {
        archetype_idxs
            .find(|&&archetype_idx| is_archetype_kept_fn(system_idx, archetype_idx, storages))
            .copied()
    }
}
