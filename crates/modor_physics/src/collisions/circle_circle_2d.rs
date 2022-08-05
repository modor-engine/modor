use crate::colliders::circle_2d::Circle2DCollider;

pub(crate) fn are_colliding(reference: &Circle2DCollider, other: &Circle2DCollider) -> bool {
    reference.center.distance(other.center) <= reference.radius + other.radius
}
