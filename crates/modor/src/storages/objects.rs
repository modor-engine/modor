use crate::storages::actions::ActionStorage;
use crate::storages::object_ids::ObjectIdStorage;
use crate::storages::ordering::OrderingStorage;
use crate::{Context, DynId, Id, Object, Objects};
use fxhash::FxHashMap;
use log::trace;
use std::any;
use std::any::{Any, TypeId};
use std::marker::PhantomData;

#[derive(Default, Debug)]
pub(crate) struct ObjectStorage {
    objects: FxHashMap<TypeId, DynObjects>,
    ordering: OrderingStorage,
}

impl ObjectStorage {
    pub(crate) fn get<T>(&self) -> crate::Result<&Objects<T>>
    where
        T: Object,
    {
        self.objects
            .get(&TypeId::of::<T>())
            .map_or(Objects::DEFAULT, DynObjects::get)
            .checked()
    }

    pub(crate) fn get_mut<T>(&mut self) -> crate::Result<&mut Objects<T>>
    where
        T: Object,
    {
        self.objects
            .entry(TypeId::of::<T>())
            .or_insert_with(|| DynObjects::new(Objects::<T>::default()))
            .get_mut()
            .checked_mut()
    }

    pub(crate) fn add_object<T>(&mut self, id: Id<T>, object: T)
    where
        T: Object,
    {
        self.ordering.register::<T>();
        self.objects
            .entry(TypeId::of::<T>())
            .or_insert_with(|| DynObjects::new(Objects::<T>::default()))
            .get_mut::<T>()
            .add(object, id);
    }

    pub(crate) fn delete_object(&mut self, id: DynId) {
        if let Some(objects) = self.objects.get_mut(&id.object_type_id) {
            objects.delete(id);
        }
    }

    pub(crate) fn lock<T>(
        &mut self,
        f: impl FnOnce(&mut Self, &mut Objects<T>) -> crate::Result<()>,
    ) -> crate::Result<()>
    where
        T: Object,
    {
        let mut objects = self.get_mut::<T>()?.lock()?;
        let result = f(self, &mut objects);
        self.dyn_objects(TypeId::of::<T>())
            .get_mut()
            .unlock(&mut objects);
        result
    }

    #[allow(clippy::needless_collect)]
    pub(crate) fn update(&mut self, object_ids: &mut ObjectIdStorage, actions: &mut ActionStorage) {
        trace!("object update started");
        for type_id in self.ordering.sorted_types() {
            let mut objects = self
                .dyn_objects(type_id)
                .lock()
                .expect("internal error: object type already taken");
            objects.update(self, object_ids, actions);
            trace!("`{}` objects updated", objects.type_name);
            self.dyn_objects(type_id).unlock(objects);
        }
        trace!("object update finished");
    }

    fn dyn_objects(&mut self, type_id: TypeId) -> &mut DynObjects {
        self.objects
            .get_mut(&type_id)
            .expect("internal error: missing object type")
    }
}

#[derive(Debug)]
struct DynObjects {
    type_name: &'static str,
    objects: Box<dyn Any>,
    lock_fn: fn(&mut Self) -> crate::Result<DynObjects>,
    unlock_fn: fn(&mut Self, DynObjects),
    update_fn: fn(&mut Self, &mut ObjectStorage, &mut ObjectIdStorage, &mut ActionStorage),
    delete_fn: fn(&mut Self, DynId),
}

impl DynObjects {
    fn new<T>(value: Objects<T>) -> Self
    where
        T: Object,
    {
        Self {
            type_name: any::type_name::<T>(),
            objects: Box::new(value),
            lock_fn: |self_| {
                let objects = self_.get_mut::<T>().lock()?;
                Ok(Self::new(objects))
            },
            unlock_fn: |self_, mut objects| self_.get_mut::<T>().unlock(objects.get_mut::<T>()),
            update_fn: |self_, objects, object_ids, actions| {
                if T::IS_UPDATE_ENABLED {
                    self_.get_mut::<T>().update(&mut Context {
                        objects,
                        object_ids,
                        actions,
                        self_id: None,
                        phantom: PhantomData,
                    });
                }
            },
            delete_fn: |self_, id| {
                self_.get_mut::<T>().delete(
                    id.typed()
                        .expect("internal error: wrong type used when deleting object"),
                );
            },
        }
    }

    #[inline]
    fn get<T>(&self) -> &Objects<T>
    where
        T: Object,
    {
        self.objects
            .downcast_ref()
            .expect("internal error: wrong type used when accessing objects immutably")
    }

    #[inline]
    fn get_mut<T>(&mut self) -> &mut Objects<T>
    where
        T: Object,
    {
        self.objects
            .downcast_mut()
            .expect("internal error: wrong type used when accessing objects mutably")
    }

    fn lock(&mut self) -> crate::Result<Self> {
        (self.lock_fn)(self)
    }

    fn unlock(&mut self, objects: Self) {
        (self.unlock_fn)(self, objects);
    }

    fn update(
        &mut self,
        objects: &mut ObjectStorage,
        object_ids: &mut ObjectIdStorage,
        actions: &mut ActionStorage,
    ) {
        (self.update_fn)(self, objects, object_ids, actions);
    }

    fn delete(&mut self, id: DynId) {
        (self.delete_fn)(self, id);
    }
}
