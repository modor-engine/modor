use crate::colliders::circle_2d::Circle2DCollider;
use crate::collisions::CollisionDetails;
use modor_math::Vec2;

pub(crate) fn are_colliding(reference: &Circle2DCollider, other: &Circle2DCollider) -> bool {
    reference.center.distance(other.center) <= reference.radius + other.radius
}

pub(crate) fn collision(
    reference: &Circle2DCollider,
    other: &Circle2DCollider,
) -> Option<CollisionDetails> {
    let center_distance = reference.center.distance(other.center);
    let radius_sum = reference.radius + other.radius;
    if center_distance <= radius_sum {
        let penetration_direction = (other.center - reference.center)
            .with_magnitude(1.)
            .unwrap_or(Vec2::X);
        let penetration_magnitude = radius_sum - center_distance;
        Some(CollisionDetails {
            penetration: (penetration_direction * penetration_magnitude).with_z(0.),
            contact_centroid: if reference.radius < other.radius
                && center_distance + reference.radius < other.radius
            {
                reference.center.with_z(0.)
            } else if other.radius < reference.radius
                && center_distance + other.radius < reference.radius
            {
                other.center.with_z(0.)
            } else {
                (reference.center
                    + penetration_direction * (reference.radius - penetration_magnitude / 2.))
                    .with_z(0.)
            },
        })
    } else {
        None
    }
}
