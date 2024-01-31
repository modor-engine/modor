use crate::{DynId, Id, Object};
use derivative::Derivative;
use fxhash::FxHashMap;
use std::any::TypeId;

#[derive(Default, Debug)]
pub(crate) struct ObjectIdStorage {
    object_ids: FxHashMap<TypeId, ObjectIds>,
}

impl ObjectIdStorage {
    pub(crate) fn reserve<T>(&mut self) -> ReservedObjectId<T>
    where
        T: Object,
    {
        self.object_ids
            .entry(TypeId::of::<T>())
            .or_insert_with(ObjectIds::new::<T>)
            .reserve()
    }

    pub(crate) fn delete(&mut self, id: DynId) {
        if let Some(ids) = self.object_ids.get_mut(&id.object_type_id) {
            ids.delete(id);
        }
    }
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""), Debug(bound = ""))]
pub(crate) enum ReservedObjectId<T> {
    New(Id<T>),
    Existing(Id<T>),
}

impl<T> ReservedObjectId<T> {
    pub(crate) fn id(self) -> Id<T> {
        match self {
            Self::New(id) | Self::Existing(id) => id,
        }
    }
}

#[derive(Debug)]
struct ObjectIds {
    is_singleton: bool,
    is_singleton_existing: bool,
    next_index: usize,
    generation_ids: Vec<u64>,
    free_indexes: Vec<usize>,
}

impl ObjectIds {
    fn new<T>() -> Self
    where
        T: Object,
    {
        Self {
            is_singleton: T::IS_SINGLETON,
            is_singleton_existing: false,
            next_index: 0,
            generation_ids: vec![],
            free_indexes: vec![],
        }
    }

    fn reserve<T>(&mut self) -> ReservedObjectId<T>
    where
        T: Object,
    {
        if self.is_singleton_existing {
            return ReservedObjectId::Existing(Id::new(0, self.generation_ids[0]));
        }
        let id = if let Some(index) = self.free_indexes.pop() {
            self.generation_ids[index] += 1;
            Id::new(index, self.generation_ids[index])
        } else {
            let index = self.next_index;
            self.next_index += 1;
            self.generation_ids.push(0);
            Id::new(index, 0)
        };
        if self.is_singleton {
            self.is_singleton_existing = true;
        }
        ReservedObjectId::New(id)
    }

    fn delete(&mut self, id: DynId) {
        self.free_indexes.push(id.index);
        if self.is_singleton {
            self.is_singleton_existing = false;
        }
    }
}
