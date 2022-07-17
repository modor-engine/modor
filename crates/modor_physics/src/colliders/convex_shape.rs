use crate::colliders::{utils, CollisionDetails};
use modor_math::Vec3;
use smallvec::SmallVec;

pub(crate) struct ConvexShapeProperties {
    pub(crate) axes: SmallVec<[Vec3; 4]>,
    pub(crate) points: SmallVec<[Vec3; 4]>,
}

pub(crate) trait ConvexShape: Sized {
    fn properties(&self) -> ConvexShapeProperties;

    fn check_collision<O>(&self, other: &O) -> Option<CollisionDetails>
    where
        O: ConvexShape,
    {
        let ref_properties = self.properties();
        let other_properties = other.properties();
        let mut best_collision = None;
        for axis in ref_properties.axes {
            let collision = collision(axis, &ref_properties.points, &other_properties.points)?;
            if collision.is_more_accurate_than(&best_collision) {
                best_collision = Some(collision);
            }
        }
        for axis in other_properties.axes {
            let collision = collision(axis, &other_properties.points, &ref_properties.points)?;
            if collision.is_more_accurate_than(&best_collision) {
                best_collision = Some(collision.to_opposite());
            }
        }
        best_collision
    }
}

fn collision(axis: Vec3, ref_points: &[Vec3], other_points: &[Vec3]) -> Option<CollisionDetails> {
    let ref_min = min_projection_factor(axis, ref_points);
    let ref_max = max_projection_factor(axis, ref_points);
    let other_min = min_projection_factor(axis, other_points);
    let other_max = max_projection_factor(axis, other_points);
    let ref_min_inside = utils::is_between(ref_min, other_min, other_max);
    let ref_max_inside = utils::is_between(ref_max, other_min, other_max);
    let other_min_inside = utils::is_between(other_min, ref_min, ref_max);
    let other_max_inside = utils::is_between(other_max, ref_min, ref_max);
    if other_min_inside && other_max_inside {
        Some(full_collision(axis, other_min, other_max, ref_min, ref_max).to_opposite())
    } else if ref_min_inside && ref_max_inside {
        Some(full_collision(axis, ref_min, ref_max, other_min, other_max))
    } else if other_min_inside && ref_min_inside {
        Some(partial_collision(axis, ref_min, other_min))
    } else if other_min_inside && ref_max_inside {
        Some(partial_collision(axis, ref_max, other_min))
    } else if other_max_inside && ref_min_inside {
        Some(partial_collision(axis, ref_min, other_max))
    } else if other_max_inside && ref_max_inside {
        Some(partial_collision(axis, ref_max, other_max))
    } else {
        None
    }
}

fn partial_collision(axis: Vec3, ref_factor: f32, other_factor: f32) -> CollisionDetails {
    CollisionDetails {
        penetration_depth: axis * (other_factor - ref_factor),
    }
}

fn full_collision(
    axis: Vec3,
    inner_factor1: f32,
    inner_factor2: f32,
    outer_factor1: f32,
    outer_factor2: f32,
) -> CollisionDetails {
    let (diff1, diff2) = if utils::is_between(inner_factor1, outer_factor1, inner_factor2) {
        (outer_factor1 - inner_factor1, outer_factor2 - inner_factor2)
    } else {
        (outer_factor2 - inner_factor1, outer_factor1 - inner_factor2)
    };
    let inner_outer_diff = if utils::is_almost_eq(diff1, 0.) && utils::is_almost_eq(diff2, 0.) {
        f32::EPSILON
    } else if utils::is_almost_eq(diff1, 0.) {
        f32::EPSILON * -1. * diff2 / diff2.abs()
    } else if utils::is_almost_eq(diff2, 0.) {
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
    CollisionDetails {
        penetration_depth: axis * (inner_diff + inner_outer_diff),
    }
}

fn min_projection_factor(axis: Vec3, points: &[Vec3]) -> f32 {
    points
        .iter()
        .map(|&p| projection_factor(axis, p))
        .fold(f32::INFINITY, |a, b| a.min(b))
}

fn max_projection_factor(axis: Vec3, points: &[Vec3]) -> f32 {
    points
        .iter()
        .map(|&p| projection_factor(axis, p))
        .fold(-f32::INFINITY, |a, b| a.max(b))
}

fn projection_factor(axis: Vec3, point: Vec3) -> f32 {
    point.dot(axis) / axis.dot(axis)
}
