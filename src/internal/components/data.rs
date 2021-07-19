use std::slice::{Iter, IterMut};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub(crate) struct ComponentReadGuard<'a, C>(RwLockReadGuard<'a, Vec<Vec<C>>>);

impl<'a, C> ComponentReadGuard<'a, C> {
    pub(super) fn new(guard: RwLockReadGuard<'a, Vec<Vec<C>>>) -> Self {
        Self(guard)
    }

    pub(crate) fn archetype_iter(&self, archetype_idx: usize) -> Option<Iter<'_, C>> {
        self.0.get(archetype_idx).map(|c| c.iter())
    }
}

pub(crate) struct ComponentWriteGuard<'a, C>(RwLockWriteGuard<'a, Vec<Vec<C>>>);

impl<'a, C> ComponentWriteGuard<'a, C> {
    pub(super) fn new(guard: RwLockWriteGuard<'a, Vec<Vec<C>>>) -> Self {
        Self(guard)
    }

    pub(crate) fn archetype_iter_mut(&mut self, archetype_idx: usize) -> Option<IterMut<'_, C>> {
        self.0.get_mut(archetype_idx).map(|c| c.iter_mut())
    }
}
