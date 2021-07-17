use crate::internal::entities::data::EntityLocation;
use std::any::Any;
use std::slice::{Iter, IterMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

// TODO: add tests
// TODO: split file

#[derive(Default)]
pub(crate) struct ComponentFacade {
    components: Vec<Box<dyn ComponentArchetypes>>,
}

impl ComponentFacade {
    pub(super) fn create_type<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        self.components
            .push(Box::new(RwLock::new(Vec::<Vec<C>>::new())))
    }

    pub(super) fn delete_archetype(&mut self, type_idx: usize, archetype_idx: usize) {
        self.components[type_idx].delete_archetype(archetype_idx);
    }

    pub(crate) fn read_components<C>(&self, type_idx: usize) -> ComponentReadGuard<'_, C>
    where
        C: Any,
    {
        ComponentReadGuard(
            self.components[type_idx]
                .as_any()
                .downcast_ref::<RwLock<Vec<Vec<C>>>>()
                .expect("internal error: invalid component type used when reading components")
                .read()
                .expect("internal error: lock poisoned when reading components"),
        )
    }

    pub(crate) fn write_components<C>(&self, type_idx: usize) -> ComponentWriteGuard<'_, C>
    where
        C: Any,
    {
        ComponentWriteGuard(
            self.components[type_idx]
                .as_any()
                .downcast_ref::<RwLock<Vec<Vec<C>>>>()
                .expect("internal error: invalid component type used when writing components")
                .write()
                .expect("internal error: lock poisoned when writing components"),
        )
    }

    pub(super) fn exists<C>(&mut self, type_idx: usize, location: EntityLocation) -> bool
    where
        C: Any,
    {
        let components = self.retrieve_components_mut::<C>(type_idx);
        components
            .get(location.archetype_idx)
            .and_then(|c| c.get(location.entity_pos))
            .is_some()
    }

    pub(super) fn add<C>(&mut self, type_idx: usize, archetype_idx: usize, component: C)
    where
        C: Any,
    {
        let components = self.retrieve_components_mut(type_idx);
        (components.len()..=archetype_idx).for_each(|_| components.push(Vec::new()));
        components[archetype_idx].push(component);
    }

    pub(super) fn replace<C>(&mut self, type_idx: usize, location: EntityLocation, component: C)
    where
        C: Any,
    {
        let components = self.retrieve_components_mut(type_idx);
        components[location.archetype_idx][location.entity_pos] = component;
    }

    pub(super) fn move_(
        &mut self,
        type_idx: usize,
        src_location: EntityLocation,
        dst_archetype_idx: usize,
    ) {
        self.components[type_idx].move_(src_location, dst_archetype_idx);
    }

    pub(super) fn delete(&mut self, type_idx: usize, location: EntityLocation) {
        self.components[type_idx].delete(location);
    }

    fn retrieve_components_mut<C>(&mut self, type_idx: usize) -> &mut Vec<Vec<C>>
    where
        C: Any,
    {
        self.components[type_idx]
            .as_any_mut()
            .downcast_mut::<RwLock<Vec<Vec<C>>>>()
            .expect("internal error: invalid component type used when adding component")
            .get_mut()
            .expect("internal error: lock poisoned when adding component")
    }
}

trait ComponentArchetypes: Any + Sync + Send {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn delete_archetype(&mut self, archetype_idx: usize);

    fn move_(&mut self, src_location: EntityLocation, dst_archetype_idx: usize);

    fn delete(&mut self, location: EntityLocation);
}

impl<C> ComponentArchetypes for RwLock<Vec<Vec<C>>>
where
    C: Any + Sync + Send,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn delete_archetype(&mut self, archetype_idx: usize) {
        let components = self
            .get_mut()
            .expect("internal error: lock poisoned when cleaning component archetype");
        components[archetype_idx] = Vec::new();
    }

    fn move_(&mut self, src_location: EntityLocation, dst_archetype_idx: usize) {
        let components = self
            .get_mut()
            .expect("internal error: lock poisoned when moving component");
        (components.len()..=dst_archetype_idx).for_each(|_| components.push(Vec::new()));
        let component = components[src_location.archetype_idx].swap_remove(src_location.entity_pos);
        components[dst_archetype_idx].push(component);
    }

    fn delete(&mut self, location: EntityLocation) {
        let components = self
            .get_mut()
            .expect("internal error: lock poisoned when deleting component");
        components[location.archetype_idx].swap_remove(location.entity_pos);
    }
}

pub(crate) struct ComponentReadGuard<'a, C>(RwLockReadGuard<'a, Vec<Vec<C>>>);

impl<'a, C> ComponentReadGuard<'a, C> {
    pub(crate) fn archetype_iter(&self, archetype_idx: usize) -> Option<Iter<'_, C>> {
        self.0.get(archetype_idx).map(|c| c.iter())
    }
}

pub(crate) struct ComponentWriteGuard<'a, C>(RwLockWriteGuard<'a, Vec<Vec<C>>>);

impl<'a, C> ComponentWriteGuard<'a, C> {
    pub(crate) fn archetype_iter_mut(&mut self, archetype_idx: usize) -> Option<IterMut<'_, C>> {
        self.0.get_mut(archetype_idx).map(|c| c.iter_mut())
    }
}
