use modor_math::{Vec2, Vec3};

pub(crate) mod circle_circle_2d;
pub(crate) mod convex_convex_2d;
pub(crate) mod point_point_2d;
pub(crate) mod point_segment_2d;
pub(crate) mod segment_segment_2d;
pub(crate) mod utils;

pub(crate) struct CollisionDetails {
    pub(crate) penetration: Vec3,
    pub(crate) contact_centroid: Vec3,
}

#[derive(Debug)]
pub(crate) enum Intersection2D {
    None,
    Point(Vec2),
    Segment(Vec2, Vec2),
}
