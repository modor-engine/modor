use crate::{DynId, Id, Object};
use fxhash::{FxHashMap, FxHashSet};

#[derive(Debug, Default)]
pub(crate) struct HierarchyStorage {
    parents: FxHashMap<DynId, Option<DynId>>,
    children: FxHashMap<DynId, FxHashSet<DynId>>,
}

impl HierarchyStorage {
    pub(crate) fn add<T>(&mut self, object: Id<T>, parent: Option<DynId>)
    where
        T: Object,
    {
        if let Some(parent) = parent {
            *self.parents.entry(object.into()).or_default() = Some(parent);
            self.children
                .entry(parent)
                .or_default()
                .insert(object.into());
        }
    }

    pub(crate) fn delete(&mut self, object: DynId, callback: &mut impl FnMut(DynId)) {
        if let Some(Some(parent)) = self.parents.remove(&object) {
            if let Some(parent_children) = self.children.get_mut(&parent) {
                parent_children.remove(&object);
            }
        }
        self.delete_child(object, callback);
    }

    fn delete_child(&mut self, object: DynId, callback: &mut impl FnMut(DynId)) {
        callback(object);
        self.parents.remove(&object);
        for &id in self.children.remove(&object).iter().flatten() {
            self.delete_child(id, callback);
        }
    }
}
