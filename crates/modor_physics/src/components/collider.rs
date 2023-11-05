use crate::components::collision_groups::CollisionGroupRegistry;
use crate::components::pipeline::{
    ColliderHandleRemoval, Pipeline2D, UnsynchronizedHandleDeletion,
};
use crate::{CollisionGroup, Dynamics2D, Transform2D};
use approx::AbsDiffEq;
use modor::{
    ComponentSystems, Custom, Entity, Filter, Not, Query, QuerySystemParam,
    QuerySystemParamWithLifetime, SingleMut, With,
};
use modor_math::Vec2;
use modor_resources::{ResKey, ResourceAccessor};
use rapier2d::geometry::{
    ActiveCollisionTypes, Collider, ColliderHandle, ContactManifold, InteractionGroups,
};
use rapier2d::math::{Point, Rotation};
use rapier2d::na::vector;
use rapier2d::pipeline::ActiveHooks;
use rapier2d::prelude::{nalgebra, ColliderBuilder, SharedShape};
use std::slice::Iter;
use crate::components::dynamics::BodyUpdate;

/// The collision properties of a 2D entity.
///
/// # Limits
///
/// The collisions may not be updated when only the size of the [`Transform2D`]
/// component is changed. However it is ensured the collision is detected when updating
/// the position or the rotation of the [`Transform2D`] component.
///
/// # Requirements
///
/// The component is effective only if:
/// - physics [`module`](crate::module()) is initialized
/// - [`Transform2D`] component is in the same entity
///
/// # Related components
///
/// - [`Transform2D`]
/// - [`Dynamics2D`]
///
/// # Example
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// # use modor_resources::*;
/// #
/// const RECTANGLE_GROUP: ResKey<CollisionGroup> = ResKey::new("rectangle");
/// const CIRCLE_GROUP: ResKey<CollisionGroup> = ResKey::new("circle");
///
/// App::new()
///     .with_entity(modor_physics::module())
///     .with_entity(CollisionGroup::new(RECTANGLE_GROUP, rectangle_collision_type))
///     .with_entity(CollisionGroup::new(CIRCLE_GROUP, circle_collision_type))
///     .with_entity(rectangle())
///     .with_entity(circle());
///
/// fn rectangle_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
///     if group_key == CIRCLE_GROUP {
///         CollisionType::Sensor
///     } else {
///         CollisionType::None
///     }
/// }
///
/// fn circle_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
///     // circle can still collide with rectangle because of rectangle_collision_type() definition
///     CollisionType::None
/// }
///
/// fn rectangle() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .component(Collider2D::rectangle(RECTANGLE_GROUP))
/// }
///
/// fn circle() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .component(Collider2D::circle(CIRCLE_GROUP))
/// }
/// ```
#[derive(Component, Debug)]
pub struct Collider2D {
    pub(crate) group_key: ResKey<CollisionGroup>,
    pub(crate) handle: Option<ColliderHandle>,
    pub(crate) collisions: Vec<Collision2D>,
    shape: Collider2DShape,
}

#[systems]
impl Collider2D {
    /// Creates a new rectangle collider.
    pub fn rectangle(group_key: ResKey<CollisionGroup>) -> Self {
        Self::new(group_key, Collider2DShape::Rectangle)
    }

    /// Creates a new circle collider.
    ///
    /// The radius of the circle is the smallest [`Transform2D`] size coordinate of the entity.
    pub fn circle(group_key: ResKey<CollisionGroup>) -> Self {
        Self::new(group_key, Collider2DShape::Circle)
    }

    #[run_as(action(ColliderHandleRemoval))]
    fn reset_handle_if_transform_removed(&mut self, _filter: Filter<Not<With<Transform2D>>>) {
        self.handle = None;
    }

    #[run_as(action(ColliderHandleRemoval))]
    fn reset_handle_if_dynamics_created(&mut self, dynamics: &Dynamics2D) {
        if dynamics.handle.is_none() {
            self.handle = None;
        }
    }

    #[run_as(action(ColliderUpdate))]
    fn update_pipeline(
        &mut self,
        transform: &Transform2D,
        dynamics: Option<&Dynamics2D>,
        entity: Entity<'_>,
        mut pipeline: SingleMut<'_, '_, Pipeline2D>,
        collision_groups: Custom<ResourceAccessor<'_, CollisionGroup>>,
    ) {
        let pipeline = pipeline.get_mut();
        let group = collision_groups.get(self.group_key);
        let interactions = group.map_or_else(InteractionGroups::none, |group| group.interactions);
        let data = ColliderUserData::new(entity.id(), group.map_or(u64::MAX, |group| group.id));
        if let Some(collider) = self.handle.and_then(|handle| pipeline.collider_mut(handle)) {
            if dynamics.is_none() {
                collider.set_translation(vector![transform.position.x, transform.position.y]);
                collider.set_rotation(Rotation::new(transform.rotation));
            }
            collider.set_shape(self.shape(transform));
            collider.set_collision_groups(interactions);
            collider.user_data = data.into();
        } else {
            let collider = self.create_collider(transform, interactions, data, dynamics.is_some());
            self.handle = Some(pipeline.create_collider(collider, dynamics.and_then(|d| d.handle)));
        }
    }

    #[run_after(component(Pipeline2D))]
    fn sync_pipeline() {
        // do nothing, this system only waits for collision update
    }

    /// Returns the detected collisions.
    pub fn collisions(&self) -> &[Collision2D] {
        &self.collisions
    }

    /// Returns an iterator on colliding objects.
    ///
    /// `query` is used to retrieved the information to return for each colliding object.
    pub fn collided<'a, 'b, P>(&'a self, query: &'a Query<'b, P>) -> Collided2DIter<'a, 'b, P>
    where
        P: 'static + QuerySystemParam,
    {
        Collided2DIter {
            collisions: self.collisions.iter(),
            query,
        }
    }

    /// Returns an iterator on colliding objects from collision group with `group_key` key.
    ///
    /// `query` is used to retrieved the information to return for each colliding object.
    pub fn collided_as<'a, 'b, P>(
        &'a self,
        query: &'a Query<'b, P>,
        group_key: ResKey<CollisionGroup>,
    ) -> CollidedAs2DIter<'a, 'b, P>
    where
        P: 'static + QuerySystemParam,
    {
        CollidedAs2DIter {
            collisions: self.collisions.iter(),
            query,
            group_key,
        }
    }

    fn new(group_key: ResKey<CollisionGroup>, shape: Collider2DShape) -> Self {
        Self {
            shape,
            group_key,
            collisions: vec![],
            handle: None,
        }
    }

    fn create_collider(
        &mut self,
        transform: &Transform2D,
        interactions: InteractionGroups,
        data: ColliderUserData,
        has_dynamics: bool,
    ) -> Collider {
        let mut collider = ColliderBuilder::new(self.shape(transform))
            .collision_groups(interactions)
            .active_collision_types(ActiveCollisionTypes::all())
            .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::FILTER_INTERSECTION_PAIR)
            .user_data(data.into())
            .build();
        if !has_dynamics {
            collider.set_translation(vector![transform.position.x, transform.position.y]);
            collider.set_rotation(Rotation::new(transform.rotation));
        }
        collider
    }

    fn shape(&self, transform: &Transform2D) -> SharedShape {
        let size = transform.size;
        match self.shape {
            Collider2DShape::Rectangle => SharedShape::cuboid(size.x / 2., size.y / 2.),
            Collider2DShape::Circle => SharedShape::ball(size.x.min(size.y) / 2.),
        }
    }
}

#[derive(Action)]
pub(crate) struct ColliderUpdate(
    UnsynchronizedHandleDeletion,
    BodyUpdate,
    <CollisionGroup as ComponentSystems>::Action,
    <CollisionGroupRegistry as ComponentSystems>::Action,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Collider2DShape {
    Rectangle,
    Circle,
}

/// A collision detected on a [`Collider2D`](`Collider2D`).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Collision2D {
    /// ID of the collided entity ID.
    pub other_entity_id: usize,
    /// Collision group key of the collided entity.
    pub other_group_key: ResKey<CollisionGroup>,
    /// Penetration of the shape into the collided one in world units.
    ///
    /// Penetration vector starts at other shape edge and ends at current shape deepest point.
    pub penetration: Vec2,
    /// Position of the collision in world units.
    ///
    /// This position corresponds to the deepest point of the current shape inside the other shape.
    /// If more than two points have the same depth, then the collision position is the average
    /// of these points.
    pub position: Vec2,
}

impl Collision2D {
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn new(
        is_collider2: bool,
        other_entity_id: usize,
        other_group_key: ResKey<CollisionGroup>,
        collider: &Collider,
        manifold: &ContactManifold,
    ) -> Self {
        let max_distance = manifold.points.iter().map(|p| -p.dist).fold(0., f32::max);
        Self {
            other_entity_id,
            other_group_key,
            penetration: Vec2::new(manifold.data.normal.x, manifold.data.normal.y)
                * max_distance
                * if is_collider2 { -1. } else { 1. },
            position: manifold
                .points
                .iter()
                .filter(|d| d.dist.abs_diff_eq(&-max_distance, f32::EPSILON))
                .map(|p| if is_collider2 { p.local_p2 } else { p.local_p1 })
                .map(|p| Self::local_to_global_position(p, collider))
                .sum::<Vec2>()
                / manifold
                    .points
                    .iter()
                    .filter(|d| d.dist.abs_diff_eq(&-max_distance, 100. * f32::EPSILON))
                    .count() as f32,
        }
    }

    fn local_to_global_position(local_positions: Point<f32>, collider: &Collider) -> Vec2 {
        Vec2::new(local_positions.x, local_positions.y).with_rotation(collider.rotation().angle())
            + Vec2::new(collider.translation().x, collider.translation().y)
    }
}

/// An iterator on colliding objects.
///
/// This struct is created by [`Collider2D::collided`].
pub struct Collided2DIter<'a, 'b, P>
where
    P: 'static + QuerySystemParam,
{
    collisions: Iter<'a, Collision2D>,
    query: &'a Query<'b, P>,
}

impl<'a, 'b, P> Iterator for Collided2DIter<'a, 'b, P>
where
    P: 'static + QuerySystemParam,
{
    type Item = (
        &'a Collision2D,
        <P as QuerySystemParamWithLifetime<'a>>::ConstParam,
    );

    fn next(&mut self) -> Option<Self::Item> {
        self.collisions
            .find_map(|c| self.query.get(c.other_entity_id).map(|e| (c, e)))
    }
}

/// An iterator on colliding objects of a specific collision group.
///
/// This struct is created by [`Collider2D::collided_as`].
pub struct CollidedAs2DIter<'a, 'b, P>
where
    P: 'static + QuerySystemParam,
{
    collisions: Iter<'a, Collision2D>,
    query: &'a Query<'b, P>,
    group_key: ResKey<CollisionGroup>,
}

impl<'a, 'b, P> Iterator for CollidedAs2DIter<'a, 'b, P>
where
    P: 'static + QuerySystemParam,
{
    type Item = (
        &'a Collision2D,
        <P as QuerySystemParamWithLifetime<'a>>::ConstParam,
    );

    fn next(&mut self) -> Option<Self::Item> {
        self.collisions.find_map(|c| {
            if self.group_key == c.other_group_key {
                self.query.get(c.other_entity_id).map(|e| (c, e))
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ColliderUserData(u128);

impl ColliderUserData {
    #[allow(clippy::cast_lossless)]
    fn new(entity_id: usize, group_id: u64) -> Self {
        Self(group_id as u128 | ((entity_id as u128) << 64))
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn entity_id(self) -> usize {
        (self.0 >> 64) as usize
    }

    pub(crate) fn group_id(self) -> u64 {
        ((self.0 << 64) >> 64) as u64
    }
}

impl From<u128> for ColliderUserData {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<ColliderUserData> for u128 {
    fn from(data: ColliderUserData) -> Self {
        data.0
    }
}

#[cfg(test)]
mod user_data_tests {
    use crate::components::collider::ColliderUserData;

    #[modor_test]
    fn create() {
        let data = ColliderUserData::new(42, 78);
        assert_eq!(data.entity_id(), 42);
        assert_eq!(data.group_id(), 78);
    }
}
