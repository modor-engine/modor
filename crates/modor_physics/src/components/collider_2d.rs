use crate::{CollisionGroupIndex, Group, Transform2D};
use modor_math::Vec2;
use rapier2d::geometry::{Collider, ColliderBuilder, ColliderHandle, ContactManifold};
use rapier2d::math::{Point, Vector};
use rapier2d::prelude::InteractionGroups;
use std::marker::PhantomData;

/// The collision properties of a 2D entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform2D`](crate::Transform2D)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
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
    pub(crate) group_idx: usize,
    pub(crate) collisions: Vec<Collision2D>,
    pub(crate) handle: Option<ColliderHandle>,
    shape: Collider2DShape,
}

impl Collider2D {
    /// Creates a new rectangle collider.
    pub fn rectangle(group: impl Into<CollisionGroupIndex>) -> Self {
        Self::new(group.into(), Collider2DShape::Rectangle)
    }

    /// Creates a new circle collider.
    ///
    /// The radius of the circle is smallest size coordinate of the entity.
    pub fn circle(group: impl Into<CollisionGroupIndex>) -> Self {
        Self::new(group.into(), Collider2DShape::Circle)
    }

    /// Returns the detected collisions.
    #[must_use]
    pub fn collisions(&self) -> &[Collision2D] {
        &self.collisions
    }

    pub(crate) fn collider_builder(&self, size: Vec2, groups: &[Group]) -> ColliderBuilder {
        let builder = match self.shape {
            Collider2DShape::Rectangle => ColliderBuilder::cuboid(size.x / 2., size.y / 2.),
            Collider2DShape::Circle => ColliderBuilder::ball(size.x.abs().min(size.y.abs()) / 2.),
        };
        let group = groups.get(self.group_idx);
        builder
            .collision_groups(InteractionGroups::new(
                group.map_or(0, |g| g.membership_bits),
                group.map_or(0, |g| g.interaction_bits),
            ))
            .solver_groups(InteractionGroups::none())
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

    fn new(group: CollisionGroupIndex, shape: Collider2DShape) -> Self {
        Self {
            group_idx: group as usize,
            collisions: vec![],
            handle: None,
            shape,
        }
    }
}

/// A collision detected on a [`Collider2D`](`Collider2D`).
#[derive(Clone, Debug)]
pub struct Collision2D {
    /// ID of the collided entity ID.
    pub other_entity_id: usize,
    /// Collision group of the collided entity ID.
    pub other_entity_group: CollisionGroupIndex,
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
    phantom: PhantomData<()>,
}

impl Collision2D {
    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn create_pair(
        entity1_id: usize,
        entity2_id: usize,
        group1_idx: usize,
        group2_idx: usize,
        transform1: &Transform2D,
        transform2: &Transform2D,
        manifold: &ContactManifold,
    ) -> (Self, Self) {
        let normal = Vec2::new(manifold.data.normal.x, manifold.data.normal.y);
        (
            Self {
                other_entity_id: entity2_id,
                other_entity_group: CollisionGroupIndex::ALL[group2_idx],
                normal,
                position: manifold
                    .points
                    .iter()
                    .map(|p| Self::local_to_global_position(p.local_p1, transform1))
                    .sum::<Vec2>()
                    / manifold.points.len() as f32,
                phantom: PhantomData,
            },
            Self {
                other_entity_id: entity1_id,
                other_entity_group: CollisionGroupIndex::ALL[group1_idx],
                normal: -normal,
                position: manifold
                    .points
                    .iter()
                    .map(|p| Self::local_to_global_position(p.local_p2, transform2))
                    .sum::<Vec2>()
                    / manifold.points.len() as f32,
                phantom: PhantomData,
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
    use approx::assert_abs_diff_eq;

    const GROUP: [Group; 1] = [Group::new(0)];

    #[test]
    fn create_rectangle() {
        let collider = Collider2D::rectangle(CollisionGroupIndex::Group0);
        let mut rapier_collider = collider.collider_builder(Vec2::new(1., 2.), &GROUP).build();
        let shape = rapier_collider.shape().as_cuboid().unwrap();
        assert_abs_diff_eq!(shape.half_extents.x, 0.5);
        assert_abs_diff_eq!(shape.half_extents.y, 1.);
        collider.update_collider(Vec2::new(4., 3.), &mut rapier_collider);
        let shape = rapier_collider.shape().as_cuboid().unwrap();
        assert_abs_diff_eq!(shape.half_extents.x, 2.);
        assert_abs_diff_eq!(shape.half_extents.y, 1.5);
    }

    #[test]
    fn create_circle() {
        let collider = Collider2D::circle(CollisionGroupIndex::Group0);
        let mut rapier_collider = collider.collider_builder(Vec2::new(1., 2.), &GROUP).build();
        let shape = rapier_collider.shape().as_ball().unwrap();
        assert_abs_diff_eq!(shape.radius, 0.5);
        collider.update_collider(Vec2::new(4., 3.), &mut rapier_collider);
        let shape = rapier_collider.shape().as_ball().unwrap();
        assert_abs_diff_eq!(shape.radius, 1.5);
    }
}
