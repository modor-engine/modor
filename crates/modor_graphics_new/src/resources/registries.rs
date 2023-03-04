use fxhash::{FxHashMap, FxHashSet};
use log::error;
use modor::{Built, ChildBuilder, Component, Entity, EntityBuilder, Query};
use modor_internal::dyn_types::DynType;
use std::any;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};

// TODO: move in a dedicated crate
// TODO: move DynType in the new crate as ResourceKeyWrapper type
// TODO: add test for resource deletion and duplicated keys

pub trait ResourceKey:
    Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe
{
}

impl<T> ResourceKey for T where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe
{
}

pub trait Resource: Component {
    fn key(&self) -> &DynType;

    #[allow(unused_variables)]
    fn build_default(builder: &mut ChildBuilder<'_>) {
        // no default resource
    }
}

pub(crate) struct ResourceRegistry<R> {
    default_id: Option<usize>,
    entity_ids: FxHashMap<DynType, usize>,
    duplicated_keys: FxHashSet<DynType>,
    missing_keys: FxHashSet<DynType>,
    phantom: PhantomData<fn(R)>,
}

#[singleton]
impl<R> ResourceRegistry<R>
where
    R: Resource,
{
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            default_id: None,
            entity_ids: FxHashMap::default(),
            duplicated_keys: FxHashSet::default(),
            missing_keys: FxHashSet::default(),
            phantom: PhantomData,
        })
        .with_children(R::build_default)
    }

    #[run]
    fn update(&mut self, entity: Entity<'_>, resources: Query<'_, (&R, Entity<'_>)>) {
        self.default_id = entity.children().next().map(Entity::id);
        self.entity_ids.clear();
        for (resource, entity) in resources.iter() {
            let key = resource.key();
            let previous = self.entity_ids.insert(key.clone(), entity.id());
            if previous.is_some() {
                self.log_duplicated_key(key);
            }
        }
    }

    pub(crate) fn find<'a>(&mut self, key: &DynType, query: &'a Query<'_, &R>) -> &'a R {
        self.entity_ids
            .get(key)
            .and_then(|&i| query.get(i))
            .or_else(|| {
                self.log_missing_key(key);
                self.default_id.and_then(|i| query.get(i))
            })
            .unwrap_or_else(|| {
                panic!(
                    "internal error: not found default resource of type '{}'",
                    any::type_name::<R>()
                )
            })
    }

    fn log_duplicated_key(&mut self, key: &DynType) {
        if !self.duplicated_keys.contains(key) {
            let type_name = any::type_name::<R>();
            error!("duplicated resource '{:?}' of type '{}'", key, type_name);
            self.duplicated_keys.insert(key.clone());
        }
    }

    fn log_missing_key(&mut self, key: &DynType) {
        if !self.missing_keys.contains(key) {
            let type_name = any::type_name::<R>();
            error!("not found resource '{:?}' of type '{}'", key, type_name);
            self.missing_keys.insert(key.clone());
        }
    }
}
