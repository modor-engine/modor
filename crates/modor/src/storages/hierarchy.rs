use crate::{DynId, Id, Object};
use fxhash::FxHashMap;

#[derive(Debug, Default)]
pub(crate) struct HierarchyStorage {
    parents: FxHashMap<DynId, Option<DynId>>,
    children: FxHashMap<DynId, Vec<DynId>>,
}

impl HierarchyStorage {
    pub(crate) fn add<T>(&mut self, object: Id<T>, parent: Option<DynId>)
    where
        T: Object,
    {
        if let Some(parent) = parent {
            *self.parents.entry(object.into()).or_default() = Some(parent);
            self.children.entry(parent).or_default().push(object.into());
        }
    }

    pub(crate) fn delete(&mut self, object: DynId, action: &mut impl FnMut(DynId)) {
        if let Some(Some(parent)) = self.parents.remove(&object) {
            if let Some(parent_children) = self.children.get_mut(&parent) {
                if let Some(pos) = parent_children.iter().position(|&i| i == object) {
                    parent_children.swap_remove(pos);
                }
            }
        }
        self.delete_child(object, action);
    }

    fn delete_child(&mut self, object: DynId, action: &mut impl FnMut(DynId)) {
        action(object);
        self.parents.remove(&object);
        for &id in self.children.remove(&object).iter().flatten() {
            self.delete(id, action);
        }
    }
}
