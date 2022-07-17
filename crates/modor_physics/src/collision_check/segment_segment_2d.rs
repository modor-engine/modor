use crate::colliders::utils;
use modor_math::Vec2;

#[derive(Debug)]
pub(crate) enum Segments2DIntersection {
    None,
    Point(Vec2),
    Segment(Vec2, Vec2),
}

pub(crate) fn intersection(
    segment1: (Vec2, Vec2),
    segment2: (Vec2, Vec2),
) -> Segments2DIntersection {
    // https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line_segment
    let segment1_vec = segment1.0 - segment1.1;
    let segment2_vec = segment2.0 - segment2.1;
    let denominator = segment1_vec.x * segment2_vec.y - segment1_vec.y * segment2_vec.x;
    if utils::is_almost_eq(denominator, 0.) {
        parallel_segments_intersection(segment1, segment2)
    } else {
        let segments_vec = segment1.0 - segment2.0;
        let t = (segments_vec.x * segment2_vec.y - segments_vec.y * segment2_vec.x) / denominator;
        let u = (segments_vec.x * segment1_vec.y - segments_vec.y * segment1_vec.x) / denominator;
        if t >= 0. && t <= 1. && u >= 0. && u <= 1. {
            Segments2DIntersection::Point(segment1.0 + t * (segment1.1 - segment1.0))
        } else {
            Segments2DIntersection::None
        }
    }
}

fn parallel_segments_intersection(
    segment1: (Vec2, Vec2),
    segment2: (Vec2, Vec2),
) -> Segments2DIntersection {
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
        Segments2DIntersection::Segment(segment1.0, segment1.1)
    } else if segment2_inside.0 && segment2_inside.1 {
        Segments2DIntersection::Segment(segment2.0, segment2.1)
    } else if segment1_inside.0 && segment2_inside.0 {
        Segments2DIntersection::Segment(segment1.0, segment2.0)
    } else if segment1_inside.0 && segment2_inside.1 {
        Segments2DIntersection::Segment(segment1.0, segment2.1)
    } else if segment1_inside.1 && segment2_inside.0 {
        Segments2DIntersection::Segment(segment1.1, segment2.0)
    } else if segment1_inside.1 && segment2_inside.1 {
        Segments2DIntersection::Segment(segment1.1, segment2.1)
    } else {
        Segments2DIntersection::None
    }
}

fn vec_projection(axis: Vec2, point: Vec2) -> f32 {
    point.dot(axis)
}
