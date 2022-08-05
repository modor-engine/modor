use crate::collisions::Intersection2D;
use approx::AbsDiffEq;
use modor_math::Vec2;

pub(crate) fn intersection(point1: Vec2, point2: Vec2) -> Intersection2D {
    if point1.abs_diff_eq(&point2, f32::EPSILON) {
        Intersection2D::Point(point1)
    } else {
        Intersection2D::None
    }
}
