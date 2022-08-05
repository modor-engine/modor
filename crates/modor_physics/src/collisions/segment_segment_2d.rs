use crate::collisions::{point_point_2d, point_segment_2d, utils, Intersection2D};
use approx::AbsDiffEq;
use modor_math::Vec2;

pub(crate) fn intersection(segment1: (Vec2, Vec2), segment2: (Vec2, Vec2)) -> Intersection2D {
    // https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line_segment
    let segment1_vec = segment1.0 - segment1.1;
    let segment2_vec = segment2.0 - segment2.1;
    let denominator = segment1_vec.x * segment2_vec.y - segment1_vec.y * segment2_vec.x;
    if denominator.abs_diff_eq(&0., f32::EPSILON) {
        match (segment_is_point(segment1), segment_is_point(segment2)) {
            (true, true) => point_point_2d::intersection(segment1.0, segment2.1),
            (true, false) => point_segment_2d::intersection(segment1.0, segment2),
            (false, true) => point_segment_2d::intersection(segment2.0, segment1),
            (false, false) => parallel_segments_intersection(segment1, segment2),
        }
    } else {
        let segments_vec = segment1.0 - segment2.0;
        let t = (segments_vec.x * segment2_vec.y - segments_vec.y * segment2_vec.x) / denominator;
        let u = (segments_vec.x * segment1_vec.y - segments_vec.y * segment1_vec.x) / denominator;
        if t >= 0. && t <= 1. && u >= 0. && u <= 1. {
            Intersection2D::Point(segment1.0 + t * (segment1.1 - segment1.0))
        } else {
            Intersection2D::None
        }
    }
}

fn parallel_segments_intersection(
    segment1: (Vec2, Vec2),
    segment2: (Vec2, Vec2),
) -> Intersection2D {
    // TODO: use cross product instead => https://lucidar.me/fr/mathematics/check-if-a-point-belongs-on-a-line-segment/
    let distance = (segment1.1.x - segment1.0.x) * (segment2.0.y - segment1.0.y)
        - (segment1.1.y - segment1.0.y) * (segment2.0.x - segment1.0.x);
    if !distance.abs_diff_eq(&0., f32::EPSILON) {
        return Intersection2D::None;
    }
    let direction = segment1.0 - segment1.1;
    let segment1_proj = (
        vec_projection(direction, segment1.0),
        vec_projection(direction, segment1.1),
    );
    let segment2_proj = (
        vec_projection(direction, segment2.0),
        vec_projection(direction, segment2.1),
    );
    let segment1_inside = (
        utils::is_between(segment1_proj.0, segment2_proj.0, segment2_proj.1),
        utils::is_between(segment1_proj.0, segment2_proj.0, segment2_proj.1),
    );
    let segment2_inside = (
        utils::is_between(segment2_proj.0, segment1_proj.0, segment1_proj.1),
        utils::is_between(segment2_proj.0, segment1_proj.0, segment1_proj.1),
    );
    if segment1_inside.0 && segment1_inside.1 {
        Intersection2D::Segment(segment1.0, segment1.1)
    } else if segment2_inside.0 && segment2_inside.1 {
        Intersection2D::Segment(segment2.0, segment2.1)
    } else if segment1_inside.0 && segment2_inside.0 {
        Intersection2D::Segment(segment1.0, segment2.0)
    } else if segment1_inside.0 && segment2_inside.1 {
        Intersection2D::Segment(segment1.0, segment2.1)
    } else if segment1_inside.1 && segment2_inside.0 {
        Intersection2D::Segment(segment1.1, segment2.0)
    } else if segment1_inside.1 && segment2_inside.1 {
        Intersection2D::Segment(segment1.1, segment2.1)
    } else {
        Intersection2D::None
    }
}

fn vec_projection(axis: Vec2, point: Vec2) -> f32 {
    point.dot(axis)
}

fn segment_is_point(segment: (Vec2, Vec2)) -> bool {
    segment.0.abs_diff_eq(&segment.1, f32::EPSILON)
}
