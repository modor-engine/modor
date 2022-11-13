use crate::utils::UserData;
use crate::{CollisionGroupRef, CollisionType};
use fxhash::FxHashMap;
use modor_internal::dyn_traits::{DynDebug, DynHash, DynPartialEq};
use modor_internal::ti_vec::TiVecSafeOperations;
use rapier2d::prelude::{Group, PairFilterContext, PhysicsHooks, SolverFlags};
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::{RefUnwindSafe, UnwindSafe};
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct CollisionGroupStorage {
    idxs: FxHashMap<CollisionGroupKey, CollisionGroupIdx>,
    collision_types: TiVec<CollisionGroupIdx, TiVec<CollisionGroupIdx, CollisionType>>,
    group_filters: TiVec<CollisionGroupIdx, Group>,
}

impl CollisionGroupStorage {
    pub(super) fn group_filter(&self, group_idx: CollisionGroupIdx) -> Group {
        self.group_filters
            .get(group_idx)
            .copied()
            .unwrap_or(Group::NONE)
    }

    pub(super) fn register(&mut self, group_key: &CollisionGroupKey) -> CollisionGroupIdx {
        if let Some(&idx) = self.idxs.get(group_key) {
            idx
        } else {
            let next_idx = self.idxs.len().into();
            self.idxs.insert(group_key.clone(), next_idx);
            self.group_filters.push(Group::NONE);
            for (key, &idx) in &self.idxs {
                let collision_type = group_key
                    .collision_type(key)
                    .max(key.collision_type(group_key));
                if collision_type != CollisionType::None {
                    *self
                        .collision_types
                        .get_mut_or_create(Self::max_idx(idx, next_idx))
                        .get_mut_or_create(Self::min_idx(idx, next_idx)) = collision_type;
                    self.group_filters[idx] |= next_idx.group_membership();
                    self.group_filters[next_idx] |= idx.group_membership();
                }
            }
            next_idx
        }
    }

    fn min_idx(idx1: CollisionGroupIdx, idx2: CollisionGroupIdx) -> CollisionGroupIdx {
        idx1.0.min(idx2.0).into()
    }

    fn max_idx(idx1: CollisionGroupIdx, idx2: CollisionGroupIdx) -> CollisionGroupIdx {
        idx1.0.max(idx2.0).into()
    }

    fn collision_type(&self, context: &PairFilterContext<'_>) -> CollisionType {
        let user_data1: UserData = context.colliders[context.collider1].user_data.into();
        let user_data2: UserData = context.colliders[context.collider2].user_data.into();
        let group1_idx = user_data1.collision_group_idx();
        let group2_idx = user_data2.collision_group_idx();
        self.collision_types
            .get(Self::max_idx(group1_idx, group2_idx))
            .and_then(|c| c.get(Self::min_idx(group1_idx, group2_idx)).copied())
            .unwrap_or(CollisionType::None)
    }
}

impl PhysicsHooks for CollisionGroupStorage {
    fn filter_contact_pair(&self, context: &PairFilterContext<'_>) -> Option<SolverFlags> {
        match self.collision_type(context) {
            CollisionType::None => None,
            CollisionType::Sensor | CollisionType::Impulse => Some(SolverFlags::COMPUTE_IMPULSES),
        }
    }
}

idx_type!(pub(crate) CollisionGroupIdx);

impl CollisionGroupIdx {
    pub(super) fn group_membership(self) -> Group {
        (1 << (self.0 % 32)).into()
    }
}

#[doc(hidden)]
pub(crate) trait DynCollisionGroupKey:
    Sync
    + Send
    + UnwindSafe
    + RefUnwindSafe
    + DynCollisionGroupKeyClone
    + DynPartialEq
    + DynHash
    + DynDebug
{
    fn collision_type(&self, other: &dyn DynCollisionGroupKey) -> CollisionType;
}

impl<T> DynCollisionGroupKey for T
where
    T: CollisionGroupRef,
{
    fn collision_type(&self, other: &dyn DynCollisionGroupKey) -> CollisionType {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(CollisionType::None, |t| self.collision_type(t))
    }
}

dyn_clone_trait!(pub(crate) DynCollisionGroupKeyClone, DynCollisionGroupKey);

pub(crate) struct CollisionGroupKey(Box<dyn DynCollisionGroupKey>);

impl CollisionGroupKey {
    pub(crate) fn new(texture_ref: impl DynCollisionGroupKey) -> Self {
        Self(Box::new(texture_ref))
    }

    pub(crate) fn collision_type(&self, other: &Self) -> CollisionType {
        self.0.collision_type(other.0.as_ref())
    }
}

impl Clone for CollisionGroupKey {
    fn clone(&self) -> Self {
        Self(self.0.as_ref().dyn_clone())
    }
}

impl PartialEq for CollisionGroupKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.dyn_partial_eq(other.0.as_dyn_partial_eq())
    }
}

impl Eq for CollisionGroupKey {}

impl Hash for CollisionGroupKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.dyn_hash(state);
    }
}

impl Debug for CollisionGroupKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.dyn_fmt(f)
    }
}
