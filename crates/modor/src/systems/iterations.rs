use super::context::Storages;
use crate::storages::archetypes::ArchetypeIdx;
use crate::ArchetypeFilterFn;
use std::any::TypeId;
use std::slice::Iter;

#[derive(Clone)]
pub(crate) struct FilteredArchetypeIdxIter<'a> {
    pub(crate) archetype_idxs: Iter<'a, ArchetypeIdx>,
    pub(crate) archetype_filter_fn: ArchetypeFilterFn,
    pub(crate) storages: Storages<'a>,
}

impl Iterator for FilteredArchetypeIdxIter<'_> {
    type Item = ArchetypeIdx;

    fn next(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            &mut self.archetype_idxs,
            self.archetype_filter_fn,
            self.storages,
        )
    }
}

impl DoubleEndedIterator for FilteredArchetypeIdxIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        Self::next_idx(
            (&mut self.archetype_idxs).rev(),
            self.archetype_filter_fn,
            self.storages,
        )
    }
}

impl FilteredArchetypeIdxIter<'_> {
    fn next_idx<'a, I>(
        archetype_idxs: I,
        is_archetype_kept_fn: fn(&[TypeId]) -> bool,
        storages: Storages<'a>,
    ) -> Option<ArchetypeIdx>
    where
        I: Iterator<Item = &'a ArchetypeIdx>,
    {
        for &archetype_idx in archetype_idxs {
            let archetype_type_ids = storages.archetypes.type_ids(archetype_idx);
            if is_archetype_kept_fn(archetype_type_ids) {
                return Some(archetype_idx);
            }
        }
        None
    }
}
