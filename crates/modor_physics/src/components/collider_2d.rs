use crate::storages_2d::collision_groups::{CollisionGroupIdx, CollisionGroupKey};
use crate::{CollisionGroupRef, Transform2D};
use modor_math::Vec2;
use rapier2d::geometry::{Collider, ColliderBuilder, ColliderHandle, ContactManifold};
use rapier2d::math::{Point, Vector};
use rapier2d::prelude::InteractionGroups;

/// The collision properties of a 2D entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform2D`](crate::Transform2D)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during actions**: [`PhysicsModule`](crate::PhysicsModule)
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
///
/// # Limits
///
/// The collisions may not be updated when the size of the [`Transform2D`](crate::Transform2D)
/// component is changed.<br>
/// To make sure that the collisions are updated, you have to set the rotation or the position of the
/// [`Transform2D`](crate::Transform2D) component.
#[derive(Debug, Clone)]
pub struct Collider2D {
    pub(crate) group_key: CollisionGroupKey,
    pub(crate) group_idx: Option<CollisionGroupIdx>,
    pub(crate) collisions: Vec<Collision2D>,
    pub(crate) handle: Option<ColliderHandle>,
    shape: Collider2DShape,
}

impl Collider2D {
    /// Creates a new rectangle collider.
    pub fn rectangle(group_ref: impl CollisionGroupRef) -> Self {
        Self::new(group_ref, Collider2DShape::Rectangle)
    }

    /// Creates a new circle collider.
    ///
    /// The radius of the circle is smallest size coordinate of the entity.
    pub fn circle(group_ref: impl CollisionGroupRef) -> Self {
        Self::new(group_ref, Collider2DShape::Circle)
    }

    /// Returns the detected collisions.
    #[must_use]
    pub fn collisions(&self) -> &[Collision2D] {
        &self.collisions
    }

    pub(crate) fn collider_builder(&self, size: Vec2) -> ColliderBuilder {
        let builder = match self.shape {
            Collider2DShape::Rectangle => ColliderBuilder::cuboid(size.x / 2., size.y / 2.),
            Collider2DShape::Circle => ColliderBuilder::ball(size.x.abs().min(size.y.abs()) / 2.),
        };
        builder.solver_groups(InteractionGroups::none())
    }

    pub(crate) fn update_collider(&self, size: Vec2, collider: &mut Collider) {
        let shape = collider.shape_mut();
        match self.shape {
            Collider2DShape::Rectangle => {
                shape
                    .as_cuboid_mut()
                    .expect("internal error: collider is not a cuboid")
                    .half_extents = Vector::new(size.x / 2., size.y / 2.);
            }
            Collider2DShape::Circle => {
                shape
                    .as_ball_mut()
                    .expect("internal error: collider is not a ball")
                    .radius = size.x.abs().min(size.y.abs()) / 2.;
            }
        }
    }

    fn new(group_ref: impl CollisionGroupRef, shape: Collider2DShape) -> Self {
        Self {
            group_key: CollisionGroupKey::new(group_ref),
            group_idx: None,
            collisions: vec![],
            handle: None,
            shape,
        }
    }
}

/// A collision detected on a [`Collider2D`](`Collider2D`).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Collision2D {
    /// ID of the collided entity ID.
    pub other_entity_id: usize,
    /// Normalized normal of the collision.
    pub normal: Vec2,
    /// Position of the collision in world units.
    ///
    /// This position can be different for two shapes that collide with each other, but
    /// it is guaranteed that both positions are on one edge of their respective shape.
    ///
    /// The position should be in the intersection of both shapes, but this is not always the case
    /// (e.g. when one shape is fully included in the other shape).
    pub position: Vec2,
    pub(crate) other_entity_group_key: CollisionGroupKey,
}

impl Collision2D {
    /// Returns whether the other entity belongs to the provided collision group.
    pub fn has_other_entity_group(&self, group_ref: impl CollisionGroupRef) -> bool {
        self.other_entity_group_key == CollisionGroupKey::new(group_ref)
    }

    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn create_pair(
        entity1_id: usize,
        entity2_id: usize,
        group1_key: CollisionGroupKey,
        group2_key: CollisionGroupKey,
        transform1: &Transform2D,
        transform2: &Transform2D,
        manifold: &ContactManifold,
    ) -> (Self, Self) {
        let normal = Vec2::new(manifold.data.normal.x, manifold.data.normal.y);
        (
            Self {
                other_entity_id: entity2_id,
                other_entity_group_key: group2_key,
                normal,
                position: manifold
                    .points
                    .iter()
                    .map(|p| Self::local_to_global_position(p.local_p1, transform1))
                    .sum::<Vec2>()
                    / manifold.points.len() as f32,
            },
            Self {
                other_entity_id: entity1_id,
                other_entity_group_key: group1_key,
                normal: -normal,
                position: manifold
                    .points
                    .iter()
                    .map(|p| Self::local_to_global_position(p.local_p2, transform2))
                    .sum::<Vec2>()
                    / manifold.points.len() as f32,
            },
        )
    }

    fn local_to_global_position(local_positions: Point<f32>, transform: &Transform2D) -> Vec2 {
        Vec2::new(local_positions.x, local_positions.y).with_rotation(*transform.rotation)
            + *transform.position
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Collider2DShape {
    Rectangle,
    Circle,
}

#[cfg(test)]
mod collider_2d_tests {
    use super::*;
    use crate::CollisionType;

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct CollisionGroup;

    impl CollisionGroupRef for CollisionGroup {
        // coverage: off (unreachable)
        fn collision_type(&self, _other: &Self) -> CollisionType {
            CollisionType::Sensor
        }
        // coverage: on
    }

    #[test]
    fn create_rectangle() {
        let collider = Collider2D::rectangle(CollisionGroup);
        let mut rapier_collider = collider.collider_builder(Vec2::new(1., 2.)).build();
        let shape = rapier_collider.shape().as_cuboid().unwrap();
        assert_approx_eq!(shape.half_extents.x, 0.5);
        assert_approx_eq!(shape.half_extents.y, 1.);
        collider.update_collider(Vec2::new(4., 3.), &mut rapier_collider);
        let shape = rapier_collider.shape().as_cuboid().unwrap();
        assert_approx_eq!(shape.half_extents.x, 2.);
        assert_approx_eq!(shape.half_extents.y, 1.5);
    }

    #[test]
    fn create_circle() {
        let collider = Collider2D::circle(CollisionGroup);
        let mut rapier_collider = collider.collider_builder(Vec2::new(1., 2.)).build();
        let shape = rapier_collider.shape().as_ball().unwrap();
        assert_approx_eq!(shape.radius, 0.5);
        collider.update_collider(Vec2::new(4., 3.), &mut rapier_collider);
        let shape = rapier_collider.shape().as_ball().unwrap();
        assert_approx_eq!(shape.radius, 1.5);
    }
}
