use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, EntityLocation};
use crate::Component;
use fxhash::FxHashMap;
use modor_internal::ti_vec::TiVecSafeOperations;
use std::any::{Any, TypeId};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct ComponentStorage {
    idxs: FxHashMap<TypeId, ComponentTypeIdx>,
    have_systems_loaded: TiVec<ComponentTypeIdx, bool>,
    archetypes: TiVec<ComponentTypeIdx, Box<dyn ComponentArchetypeLock>>,
    sorted_archetype_idxs: TiVec<ComponentTypeIdx, Vec<ArchetypeIdx>>,
    singleton_locations: TiVec<ComponentTypeIdx, Option<EntityLocation>>,
}

impl ComponentStorage {
    pub(crate) fn type_idx(&self, component_type: TypeId) -> Option<ComponentTypeIdx> {
        self.idxs.get(&component_type).copied()
    }

    pub(crate) fn sorted_archetype_idxs(&self, type_idx: ComponentTypeIdx) -> &[ArchetypeIdx] {
        &self.sorted_archetype_idxs[type_idx]
    }

    pub(crate) fn singleton_location(&self, type_idx: ComponentTypeIdx) -> Option<EntityLocation> {
        self.singleton_locations[type_idx]
    }

    pub(crate) fn has_systems_loaded<C>(&self) -> bool
    where
        C: Component,
    {
        self.idxs
            .get(&TypeId::of::<C>())
            .map_or(false, |&i| self.have_systems_loaded[i])
    }

    pub(crate) fn read_components<C>(&self) -> RwLockReadGuard<'_, ComponentArchetypes<C>>
    where
        C: Component,
    {
        let &type_idx = self
            .idxs
            .get(&TypeId::of::<C>())
            .expect("internal error: cannot read missing component type");
        self.archetypes[type_idx]
            .as_any()
            .downcast_ref::<RwLock<ComponentArchetypes<C>>>()
            .expect("internal error: wrong component type when reading components")
            .try_read()
            .expect("internal error: cannot read archetypes when reading components")
    }

    pub(crate) fn write_components<C>(&self) -> RwLockWriteGuard<'_, ComponentArchetypes<C>>
    where
        C: Component,
    {
        let &type_idx = self
            .idxs
            .get(&TypeId::of::<C>())
            .expect("internal error: cannot write missing component type");
        self.archetypes[type_idx]
            .as_any()
            .downcast_ref::<RwLock<ComponentArchetypes<C>>>()
            .expect("internal error: wrong component type when writing components")
            .try_write()
            .expect("internal error: cannot write archetypes when writing component")
    }

    pub(crate) fn type_idx_or_create<C>(&mut self) -> ComponentTypeIdx
    where
        C: Component,
    {
        *self.idxs.entry(TypeId::of::<C>()).or_insert_with(|| {
            self.have_systems_loaded.push(false);
            let archetype_lock = RwLock::new(ComponentArchetypes::<C>::default());
            self.archetypes.push(Box::new(archetype_lock));
            self.sorted_archetype_idxs.push(vec![]);
            self.singleton_locations.push_and_get_key(None)
        })
    }

    pub(super) fn set_systems_as_loaded<C>(&mut self) -> ComponentTypeIdx
    where
        C: Component,
    {
        let type_idx = self.type_idx_or_create::<C>();
        self.have_systems_loaded[type_idx] = true;
        type_idx
    }

    pub(super) fn add<C>(
        &mut self,
        type_idx: ComponentTypeIdx,
        location: EntityLocation,
        component: C,
        is_singleton: bool,
    ) where
        C: Component,
    {
        let archetypes = self.archetypes[type_idx]
            .as_any_mut()
            .downcast_mut::<RwLock<ComponentArchetypes<C>>>()
            .expect("internal error: wrong component type when adding component")
            .get_mut()
            .expect("internal error: cannot write archetypes when adding component");
        let archetype = archetypes.get_mut_or_create(location.idx);
        if let Some(existing_component) = archetype.get_mut(location.pos) {
            *existing_component = component;
        } else {
            archetype.push(component);
            self.add_archetype(type_idx, location.idx);
        }
        if is_singleton {
            self.singleton_locations[type_idx] = Some(location);
        }
    }

    pub(super) fn move_(
        &mut self,
        type_idx: ComponentTypeIdx,
        src_location: EntityLocation,
        dst_archetype_idx: ArchetypeIdx,
    ) {
        self.archetypes[type_idx].move_component(src_location, dst_archetype_idx);
        self.add_archetype(type_idx, dst_archetype_idx);
        if let Some(singleton_location) = self.singleton_locations[type_idx] {
            let last_archetype_location = EntityLocation {
                idx: src_location.idx,
                pos: self.archetypes[type_idx]
                    .component_count(src_location.idx)
                    .into(),
            };
            if singleton_location == src_location {
                self.singleton_locations[type_idx] = Some(EntityLocation {
                    idx: dst_archetype_idx,
                    pos: ArchetypeEntityPos::default(),
                });
            } else if singleton_location == last_archetype_location {
                self.singleton_locations[type_idx] = Some(src_location);
            }
        }
    }

    pub(super) fn delete(&mut self, type_idx: ComponentTypeIdx, location: EntityLocation) {
        self.archetypes[type_idx].delete_component(location);
        if let Some(singleton_location) = self.singleton_locations[type_idx] {
            let last_archetype_location = EntityLocation {
                idx: location.idx,
                pos: self.archetypes[type_idx]
                    .component_count(location.idx)
                    .into(),
            };
            if singleton_location == location {
                self.singleton_locations[type_idx] = None;
            } else if singleton_location == last_archetype_location {
                self.singleton_locations[type_idx] = Some(location);
            }
        }
    }

    fn add_archetype(&mut self, type_idx: ComponentTypeIdx, archetype_idx: ArchetypeIdx) {
        if let Err(pos) = self.sorted_archetype_idxs[type_idx].binary_search(&archetype_idx) {
            self.sorted_archetype_idxs[type_idx].insert(pos, archetype_idx);
        }
    }
}

trait ComponentArchetypeLock: Any + Sync + Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn component_count(&mut self, archetype_idx: ArchetypeIdx) -> usize;

    fn move_component(&mut self, src_location: EntityLocation, dst_archetype_idx: ArchetypeIdx);

    fn delete_component(&mut self, location: EntityLocation);
}

pub(crate) type ComponentArchetypes<C> = TiVec<ArchetypeIdx, TiVec<ArchetypeEntityPos, C>>;

impl<C> ComponentArchetypeLock for RwLock<ComponentArchetypes<C>>
where
    C: Any + Sync + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn component_count(&mut self, archetype_idx: ArchetypeIdx) -> usize {
        self.get_mut()
            .expect("internal error: cannot get number of components in archetype")[archetype_idx]
            .len()
    }

    fn move_component(&mut self, src_location: EntityLocation, dst_archetype_idx: ArchetypeIdx) {
        let archetypes = self
            .get_mut()
            .expect("internal error: cannot write archetypes when moving component");
        let component = archetypes[src_location.idx].swap_remove(src_location.pos);
        archetypes
            .get_mut_or_create(dst_archetype_idx)
            .push(component);
    }

    fn delete_component(&mut self, location: EntityLocation) {
        let archetypes = self
            .get_mut()
            .expect("internal error: cannot write archetypes when deleting component");
        archetypes[location.idx].swap_remove(location.pos);
    }
}

idx_type!(pub ComponentTypeIdx);
