use super::iterations::FilteredArchetypeIdxIter;
use crate::storages::actions::ActionStorage;
use crate::storages::archetypes::ArchetypeStorage;
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::entities::EntityStorage;
use crate::storages::updates::UpdateStorage;
use crate::ArchetypeFilterFn;
use std::sync::Mutex;

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemContext<'a> {
    pub(crate) archetype_filter_fn: ArchetypeFilterFn,
    pub(crate) entity_type_idx: Option<ComponentTypeIdx>,
    pub(crate) item_count: usize,
    pub(crate) storages: Storages<'a>,
}

impl SystemContext<'_> {
    pub(crate) fn filter_archetype_idx_iter(&self) -> FilteredArchetypeIdxIter<'_> {
        self.storages
            .filter_archetype_idx_iter(self.archetype_filter_fn, self.entity_type_idx)
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub(crate) struct Storages<'a> {
    pub(crate) entities: &'a EntityStorage,
    pub(crate) components: &'a ComponentStorage,
    pub(crate) archetypes: &'a ArchetypeStorage,
    pub(crate) actions: &'a ActionStorage,
    pub(crate) updates: &'a Mutex<UpdateStorage>,
}

impl Storages<'_> {
    pub(crate) fn item_count(
        &self,
        archetype_filter_fn: ArchetypeFilterFn,
        entity_type_idx: Option<ComponentTypeIdx>,
    ) -> usize {
        self.filter_archetype_idx_iter(archetype_filter_fn, entity_type_idx)
            .map(|a| self.archetypes.entity_idxs(a).len())
            .sum()
    }

    fn filter_archetype_idx_iter(
        &self,
        archetype_filter_fn: ArchetypeFilterFn,
        entity_type_idx: Option<ComponentTypeIdx>,
    ) -> FilteredArchetypeIdxIter<'_> {
        let archetype_idxs = entity_type_idx.map_or_else(
            || self.archetypes.all_sorted_idxs().iter(),
            |i| self.components.sorted_archetype_idxs(i).iter(),
        );
        FilteredArchetypeIdxIter {
            archetype_idxs,
            archetype_filter_fn,
            storages: *self,
        }
    }
}
