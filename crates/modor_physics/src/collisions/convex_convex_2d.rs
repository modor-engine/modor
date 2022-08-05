use crate::colliders::convex_2d::Convex2DCollider;
use crate::collisions::segment_segment_2d;
use crate::collisions::Intersection2D;
use crate::collisions::{utils, CollisionDetails};
use approx::AbsDiffEq;
use modor_math::Vec2;

pub(crate) fn collision(
    reference: &Convex2DCollider,
    other: &Convex2DCollider,
) -> Option<CollisionDetails> {
    Some(CollisionDetails {
        penetration: penetration(reference, other)?.with_z(0.),
        contact_centroid: contact_centroid(reference, other).with_z(0.),
    })
}

fn penetration(reference: &Convex2DCollider, other: &Convex2DCollider) -> Option<Vec2> {
    let mut smallest_penetration = None;
    for &axis in &reference.normals {
        let penetration = penetration_on_axis(axis, &reference.points, &other.points)?;
        if is_penetration_smaller(penetration, smallest_penetration) {
            smallest_penetration = Some(penetration);
        }
    }
    for &axis in &other.normals {
        let penetration = -penetration_on_axis(axis, &other.points, &reference.points)?;
        if is_penetration_smaller(penetration, smallest_penetration) {
            smallest_penetration = Some(penetration);
        }
    }
    smallest_penetration
}

fn is_penetration_smaller(penetration: Vec2, other_penetration: Option<Vec2>) -> bool {
    other_penetration
        .map(|o| penetration.magnitude() < o.magnitude())
        .unwrap_or(true)
}

fn penetration_on_axis(axis: Vec2, ref_points: &[Vec2], other_points: &[Vec2]) -> Option<Vec2> {
    let ref_min = min_projection_factor(axis, ref_points);
    let ref_max = max_projection_factor(axis, ref_points);
    let other_min = min_projection_factor(axis, other_points);
    let other_max = max_projection_factor(axis, other_points);
    let ref_min_inside = utils::is_between(ref_min, other_min, other_max);
    let ref_max_inside = utils::is_between(ref_max, other_min, other_max);
    let other_min_inside = utils::is_between(other_min, ref_min, ref_max);
    let other_max_inside = utils::is_between(other_max, ref_min, ref_max);
    if other_min_inside && other_max_inside {
        Some(-full_penetration_on_axis(
            axis, other_min, other_max, ref_min, ref_max,
        ))
    } else if ref_min_inside && ref_max_inside {
        Some(full_penetration_on_axis(
            axis, ref_min, ref_max, other_min, other_max,
        ))
    } else if other_min_inside && ref_min_inside {
        Some(partial_penetration_on_axis(axis, ref_min, other_min))
    } else if other_min_inside && ref_max_inside {
        Some(partial_penetration_on_axis(axis, ref_max, other_min))
    } else if other_max_inside && ref_min_inside {
        Some(partial_penetration_on_axis(axis, ref_min, other_max))
    } else if other_max_inside && ref_max_inside {
        Some(partial_penetration_on_axis(axis, ref_max, other_max))
    } else {
        None
    }
}

fn partial_penetration_on_axis(axis: Vec2, ref_factor: f32, other_factor: f32) -> Vec2 {
    axis * (ref_factor - other_factor)
}

fn full_penetration_on_axis(
    axis: Vec2,
    inner_factor1: f32,
    inner_factor2: f32,
    outer_factor1: f32,
    outer_factor2: f32,
) -> Vec2 {
    let (diff1, diff2) = if utils::is_between(inner_factor1, outer_factor1, inner_factor2) {
        (inner_factor1 - outer_factor1, inner_factor2 - outer_factor2)
    } else {
        (inner_factor1 - outer_factor2, inner_factor2 - outer_factor1)
    };
    let inner_outer_diff =
        if diff1.abs_diff_eq(&0., f32::EPSILON) && diff2.abs_diff_eq(&0., f32::EPSILON) {
            f32::EPSILON
        } else if diff1.abs_diff_eq(&0., f32::EPSILON) {
            f32::EPSILON * -1. * diff2 / diff2.abs()
        } else if diff2.abs_diff_eq(&0., f32::EPSILON) {
            f32::EPSILON * -1. * diff1 / diff1.abs()
        } else {
            if diff1.abs() < diff2.abs() {
                diff1
            } else {
                diff2
            }
        };
    let diff_sign = if inner_outer_diff < 0. { -1. } else { 1. };
    let inner_diff = diff_sign * (inner_factor1 - inner_factor2).abs();
    axis * (inner_diff + inner_outer_diff)
}

fn min_projection_factor(axis: Vec2, points: &[Vec2]) -> f32 {
    points
        .iter()
        .map(|&p| projection_factor(axis, p))
        .fold(f32::INFINITY, |a, b| a.min(b))
}

fn max_projection_factor(axis: Vec2, points: &[Vec2]) -> f32 {
    points
        .iter()
        .map(|&p| projection_factor(axis, p))
        .fold(-f32::INFINITY, |a, b| a.max(b))
}

fn projection_factor(axis: Vec2, point: Vec2) -> f32 {
    point.dot(axis) / axis.dot(axis)
}

fn contact_centroid(reference: &Convex2DCollider, other: &Convex2DCollider) -> Vec2 {
    let mut contact_sum = Vec2::ZERO;
    let mut contact_count = 0.;
    for &ref_segment in &reference.segments {
        for &other_segment in &other.segments {
            let intersection = segment_segment_2d::intersection(ref_segment, other_segment);
            match intersection {
                Intersection2D::Point(p) => {
                    contact_sum += p;
                    contact_count += 1.;
                }
                Intersection2D::Segment(p1, p2) => {
                    contact_sum += p1 + p2;
                    contact_count += 2.;
                }
                Intersection2D::None => {}
            }
        }
    }
    if contact_count < 0.5 {
        if reference.size.magnitude() > other.size.magnitude() {
            other.position
        } else {
            reference.position
        }
    } else {
        contact_sum / contact_count
    }
}
