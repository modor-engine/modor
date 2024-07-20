use approx::AbsDiffEq;
use modor_math::Vec2;
use rapier2d::geometry::{Collider, ContactManifold};
use rapier2d::na::Point2;

/// A detected collision.
///
/// # Examples
///
/// See [`Body2D`](crate::Body2D).
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Collision2D {
    /// Index of the collided body.
    pub other_index: usize,
    /// Index of the collision group corresponding to the collided body.
    pub other_group_index: usize,
    /// Penetration of the body into the collided one in world units.
    ///
    /// Penetration vector starts at other body edge and ends at current body deepest point.
    pub penetration: Vec2,
    /// Position of the collision in world units.
    ///
    /// This position corresponds to the deepest point of the current body inside the other body.
    /// If more than two points have the same depth, then the collision position is the average
    /// of these points.
    pub position: Vec2,
}

impl Collision2D {
    pub(crate) fn new(
        is_collider2: bool,
        other_index: usize,
        other_group_index: usize,
        collider: &Collider,
        manifold: &ContactManifold,
    ) -> Self {
        let max_distance = manifold.points.iter().map(|p| -p.dist).fold(0., f32::max);
        Self {
            other_index,
            other_group_index,
            penetration: Self::penetration(is_collider2, manifold, max_distance),
            position: Self::position(is_collider2, collider, manifold, max_distance),
        }
    }

    fn penetration(is_collider2: bool, manifold: &ContactManifold, max_distance: f32) -> Vec2 {
        Vec2::new(manifold.data.normal.x, manifold.data.normal.y)
            * max_distance
            * if is_collider2 { -1. } else { 1. }
    }

    #[allow(clippy::cast_precision_loss)]
    fn position(
        is_collider2: bool,
        collider: &Collider,
        manifold: &ContactManifold,
        max_distance: f32,
    ) -> Vec2 {
        manifold
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
                .count() as f32
    }

    fn local_to_global_position(local_positions: Point2<f32>, collider: &Collider) -> Vec2 {
        Vec2::new(local_positions.x, local_positions.y).with_rotation(collider.rotation().angle())
            + Vec2::new(collider.translation().x, collider.translation().y)
    }
}
