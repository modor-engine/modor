use super::iterations::FilteredArchetypeIdxIter;
use crate::storages::actions::ActionStorage;
use crate::storages::archetype_states::ArchetypeStateStorage;
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::entities::EntityStorage;
use crate::storages::systems::SystemIdx;
use crate::storages::updates::UpdateStorage;
use crate::{ArchetypeFilterFn, Component};
use std::any::TypeId;
use std::sync::{Mutex, RwLock};

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemContext<'a> {
    pub(crate) system_idx: Option<SystemIdx>,
    pub(crate) archetype_filter_fn: ArchetypeFilterFn,
    pub(crate) entity_type_idx: Option<ComponentTypeIdx>,
    pub(crate) item_count: usize,
    pub(crate) storages: Storages<'a>,
}

impl SystemContext<'_> {
    pub(crate) fn component_type_idx<C>(&self) -> ComponentTypeIdx
    where
        C: Component,
    {
        self.storages
            .components
            .type_idx(TypeId::of::<C>())
            .expect("internal error: component type not registered")
    }

    pub(crate) fn filter_archetype_idx_iter(&self) -> FilteredArchetypeIdxIter<'_> {
        self.storages.filter_archetype_idx_iter(
            self.system_idx,
            self.archetype_filter_fn,
            self.entity_type_idx,
        )
    }

    pub(crate) fn add_mutated_component(
        &self,
        component_type_idx: ComponentTypeIdx,
        archetype_idx: ArchetypeIdx,
    ) {
        self.storages
            .archetype_states
            .write()
            .expect("internal error: cannot add mutated component to archetype state")
            .add_mutated_component(component_type_idx, archetype_idx, self.system_idx);
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct Storages<'a> {
    pub(crate) entities: &'a EntityStorage,
    pub(crate) components: &'a ComponentStorage,
    pub(crate) archetypes: &'a ArchetypeStorage,
    pub(crate) actions: &'a ActionStorage,
    pub(crate) updates: &'a Mutex<UpdateStorage>,
    pub(crate) archetype_states: &'a RwLock<ArchetypeStateStorage>,
}

impl Storages<'_> {
    pub(crate) fn item_count(
        &self,
        system_idx: Option<SystemIdx>,
        archetype_filter_fn: ArchetypeFilterFn,
        entity_type_idx: Option<ComponentTypeIdx>,
    ) -> usize {
        self.filter_archetype_idx_iter(system_idx, archetype_filter_fn, entity_type_idx)
            .map(|a| self.archetypes.entity_idxs(a).len())
            .sum()
    }

    fn filter_archetype_idx_iter(
        &self,
        system_idx: Option<SystemIdx>,
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
            system_idx,
        }
    }
}
