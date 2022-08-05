use crate::collisions::Intersection2D;
use approx::AbsDiffEq;
use modor_math::Vec2;

pub(crate) fn intersection(point: Vec2, segment: (Vec2, Vec2)) -> Intersection2D {
    let segment_point = point - segment.0;
    let segment_segment = segment.1 - segment.0;
    if (segment_point.x * segment_segment.y - segment_point.y * segment_segment.x)
        .abs_diff_eq(&0., f32::EPSILON)
    {
        let a = segment_point.dot(segment_segment);
        let b = segment_point.dot(segment_point);
        if 0. <= a && a <= b {
            Intersection2D::Point(point)
        } else {
            Intersection2D::None
        }
    } else {
        Intersection2D::None
    }
}
