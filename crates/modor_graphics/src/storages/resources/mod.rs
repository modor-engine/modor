use fxhash::FxHashMap;
use std::any::Any;
use std::hash::Hash;
use std::iter;

macro_rules! resource_key {
    ($name:ident, $trait_name:ident) => {
        pub(crate) struct $name(Box<dyn $trait_name>);

        impl $name {
            pub(crate) fn new(resource_ref: impl $trait_name) -> Self {
                Self(Box::new(resource_ref))
            }
        }

        impl Clone for $name {
            fn clone(&self) -> Self {
                Self(self.0.as_ref().dyn_clone())
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.0.dyn_partial_eq(other.0.as_dyn_partial_eq())
            }
        }

        impl Eq for $name {}

        impl std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.dyn_hash(state);
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.dyn_fmt(f)
            }
        }

        #[allow(unreachable_pub)]
        pub trait $trait_name:
            Sync
            + Send
            + std::panic::UnwindSafe
            + std::panic::RefUnwindSafe
            + DynResourceKeyClone
            + modor_internal::dyn_traits::DynPartialEq
            + modor_internal::dyn_traits::DynHash
            + modor_internal::dyn_traits::DynDebug
        {
        }

        dyn_clone_trait!(pub DynResourceKeyClone, $trait_name);
    };
}

pub(super) struct ResourceStorage<K, R> {
    default_key: K,
    resources: FxHashMap<K, StoredResource<R>>,
}

impl<K, R> ResourceStorage<K, R>
where
    K: Any + Hash + Eq,
{
    pub(super) fn default_key(&self) -> &K {
        &self.default_key
    }

    pub(super) fn get_default(&self) -> &R {
        &self.resources[&self.default_key].resource
    }

    pub(super) fn get(&self, key: &K) -> Option<&R> {
        self.resources.get(key).map(|t| &t.resource)
    }

    pub(super) fn add(&mut self, key: K, resource: R) {
        self.resources.insert(
            key,
            StoredResource {
                resource,
                is_deleted: false,
            },
        );
    }

    pub(crate) fn remove_not_found<'a>(&mut self, existing_keys: impl Iterator<Item = &'a K>) {
        for (key, resource) in &mut self.resources {
            if key != &self.default_key {
                resource.is_deleted = true;
            }
        }
        for key in existing_keys {
            if let Some(resource) = self.resources.get_mut(key) {
                resource.is_deleted = false;
            }
        }
        self.resources.retain(|_, t| !t.is_deleted);
    }

    fn create(default_key: K, default_resource: R) -> Self
    where
        K: Clone,
    {
        Self {
            resources: iter::once((
                default_key.clone(),
                StoredResource {
                    resource: default_resource,
                    is_deleted: false,
                },
            ))
            .collect(),
            default_key,
        }
    }
}

struct StoredResource<R> {
    resource: R,
    is_deleted: bool,
}

pub(crate) mod fonts;
pub(crate) mod textures;
