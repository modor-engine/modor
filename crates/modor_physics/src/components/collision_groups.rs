use modor::{Entity, Query};
use modor_resources::{ResKey, Resource, ResourceRegistry, ResourceState};
use rapier2d::prelude::{Group, InteractionGroups};
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) type CollisionGroupRegistry = ResourceRegistry<CollisionGroup>;

/// A collision group that can be attached to a [`Collider2D`](crate::Collider2D).
///
/// # Example
///
/// See [`Collider2D`](crate::Collider2D).
#[derive(Component, Debug, Clone)]
pub struct CollisionGroup {
    pub(crate) id: u64,
    pub(crate) interactions: InteractionGroups,
    pub(crate) collision_type_fn: fn(ResKey<Self>) -> CollisionType,
    key: ResKey<Self>,
}

#[systems]
impl CollisionGroup {
    /// Creates a new collision group with a unique `key`.
    ///
    /// `collision_type_fn` expects the collision group key of an object that collides with
    /// objects belonging to the created collision group, and defines how they should behave.
    /// Note that when two objects from a different collision group collide, the greatest
    /// `CollisionType` returned by `collision_type_fn` from both groups is used.
    pub fn new(key: ResKey<Self>, collision_type_fn: fn(ResKey<Self>) -> CollisionType) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, u64::MAX, "too many `CollisionGroup` instances created");
        Self {
            key,
            collision_type_fn,
            id,
            interactions: InteractionGroups::all(),
        }
    }

    #[run]
    fn update(entity: Entity<'_>, mut query: Query<'_, &mut Self>) {
        let groups: Vec<_> = query.iter().cloned().collect();
        query
            .get_mut(entity.id())
            .expect("collision group not found")
            .update_interactions(&groups);
    }

    fn update_interactions(&mut self, groups: &[Self]) {
        self.interactions = InteractionGroups::new(
            Group::from(self.memberships()),
            Group::from(self.filter(groups)),
        );
    }

    fn memberships(&self) -> u32 {
        1 << (self.id % 32)
    }

    fn filter(&self, groups: &[Self]) -> u32 {
        groups
            .iter()
            .filter(|group| {
                (self.collision_type_fn)(group.key) != CollisionType::None
                    || (group.collision_type_fn)(self.key) != CollisionType::None
            })
            .map(Self::memberships)
            .fold(0, |a, b| a | b)
    }
}

impl Resource for CollisionGroup {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::Loaded
    }
}

/// The collision behavior that should happen between two objects.
///
/// # Examples
///
/// See [`Collider2D`](crate::Collider2D).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[non_exhaustive]
pub enum CollisionType {
    /// No collision should happen.
    #[default]
    None,
    /// Collision should happen but it doesn't produce forces.
    Sensor,
    /// Collision should happen and it produces forces. This has currently the same effect as
    /// [`CollisionType::Sensor`](CollisionType::Sensor).
    Impulse,
}
