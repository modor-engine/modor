use derivative::Derivative;
use fxhash::FxHashMap;
use modor::{Component, World};
use modor_resources::{ResKey, Resource, ResourceRegistry};
use std::collections::hash_map::Entry;
use std::hash::Hash;

#[derive(Debug, Derivative)]
#[derivative(Default(bound = ""))]
pub(crate) struct ManagedResources<K, R> {
    data: FxHashMap<K, ManagedRes<R>>,
}

impl<K, R> ManagedResources<K, R>
where
    K: Eq + Hash,
    R: Resource + Component,
{
    pub(crate) fn contains(&self, key: K) -> bool {
        self.data.contains_key(&key)
    }

    pub(crate) fn managed_key(&self, key: K) -> Option<ResKey<R>> {
        self.data.get(&key).map(|data| data.key)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&K, ResKey<R>)> + '_ {
        self.data.iter().map(|(k, v)| (k, v.key))
    }

    pub(crate) fn reset(&mut self) {
        for data in self.data.values_mut() {
            data.is_registered = false;
        }
    }

    pub(crate) fn register(&mut self, key: K) -> ResState<R> {
        match self.data.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().is_registered = true;
                ResState::Existing
            }
            Entry::Vacant(entry) => ResState::New(entry.insert(ManagedRes::new()).key),
        }
    }

    pub(crate) fn delete_not_registered(
        &mut self,
        registry: &ResourceRegistry<R>,
        world: &mut World<'_>,
    ) {
        for data in self.data.values_mut() {
            if !data.is_registered {
                if let Some(id) = registry.entity_id(data.key) {
                    world.delete_entity(id);
                }
            }
        }
        self.data.retain(|_, data| data.is_registered);
    }
}

pub(crate) enum ResState<R> {
    New(ResKey<R>),
    Existing,
}

#[derive(Debug)]
struct ManagedRes<R> {
    pub(crate) key: ResKey<R>,
    is_registered: bool,
}

impl<R> ManagedRes<R>
where
    R: Resource,
{
    fn new() -> Self {
        Self {
            key: ResKey::unique("managed-resource(modor_picking)"),
            is_registered: true,
        }
    }
}
