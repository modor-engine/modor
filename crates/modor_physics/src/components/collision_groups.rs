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
#[derive(Component)]
pub struct CollisionGroup {
    pub(crate) id: u64,
    pub(crate) interactions: InteractionGroups,
    pub(crate) collision_type_fn: Box<dyn Fn(ResKey<Self>) -> CollisionType + Sync + Send>,
    key: ResKey<Self>,
}

#[systems]
impl CollisionGroup {
    /// Creates a new collision group with a unique `key`.
    ///
    /// `collision_type_fn` expects the collision group key of an object that collides with
    /// objects belonging to the created collision group, and defines how they should behave.
    /// Note that when two objects from a different collision group collide, the
    /// [`CollisionType`] with highest priority returned by `collision_type_fn` from both groups
    /// is used.
    ///
    /// Priority of [`CollisionType`] is the following:
    /// [`CollisionType::Impulse`] > [`CollisionType::Sensor`] > [`CollisionType::None`].
    pub fn new(
        key: ResKey<Self>,
        collision_type_fn: impl Fn(ResKey<Self>) -> CollisionType + Sync + Send + 'static,
    ) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, u64::MAX, "too many `CollisionGroup` instances created");
        Self {
            key,
            collision_type_fn: Box::new(collision_type_fn),
            id,
            interactions: InteractionGroups::all(),
        }
    }

    #[run]
    fn update(entity: Entity<'_>, mut query: Query<'_, &mut Self>) {
        let group = query
            .get(entity.id())
            .expect("internal error: collision group not found");
        let interactions = InteractionGroups::new(
            Group::from(group.memberships()),
            Group::from(group.filter(query.iter())),
        );
        query
            .get_mut(entity.id())
            .expect("internal error: collision group not found")
            .interactions = interactions;
    }

    fn memberships(&self) -> u32 {
        1 << (self.id % 32)
    }

    fn filter<'a>(&self, groups: impl Iterator<Item = &'a Self>) -> u32 {
        groups
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
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[non_exhaustive]
pub enum CollisionType {
    /// No collision should happen.
    #[default]
    None,
    /// Collision should happen but it doesn't produce forces.
    Sensor,
    /// Collision should happen and it produces forces.
    Impulse(Impulse),
}

impl CollisionType {
    pub(crate) fn highest_priority(self, other: Self) -> Self {
        if let (Self::None, Self::Sensor | Self::Impulse(_)) | (Self::Sensor, Self::Impulse(_)) =
            (self, other)
        {
            other
        } else {
            self
        }
    }
}

/// Properties of a collision of type [`CollisionType::Impulse`].
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Impulse {
    /// Restitution coefficient of the collision.
    ///
    /// A coefficient of `0.0` means that the entities do not bounce off each other at all.<br>
    /// A coefficient of `1.0` means that the exit velocity magnitude is the same as the initial
    /// velocity along the contact normal.
    ///
    /// Default value is `0.0`.
    pub restitution: f32,
    /// Friction coefficient of the collision.
    ///
    /// A coefficient of `0.0` means there is no friction (i.e. objects slide completely over each
    /// other).
    ///
    /// Default value is `0.5`.
    pub friction: f32,
}

impl Default for Impulse {
    fn default() -> Self {
        Self {
            restitution: 0.,
            friction: 0.5,
        }
    }
}

impl Impulse {
    /// Creates a new impulse configuration.
    pub fn new(restitution: f32, friction: f32) -> Self {
        Self {
            restitution,
            friction,
        }
    }
}
