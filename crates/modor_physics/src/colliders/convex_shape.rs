use crate::colliders::{utils, CollisionDetails};
use crate::collision_check::segment_segment_2d;
use crate::collision_check::segment_segment_2d::Segments2DIntersection;
use modor_math::Vec3;
use smallvec::SmallVec;

#[derive(Debug)]
pub(crate) enum Surface {
    Segment(Vec3, Vec3),
    Triangle(Vec3, Vec3, Vec3),
}

pub(crate) struct ConvexShapeProperties {
    pub(crate) position: Vec3,
    pub(crate) size: Vec3,
    pub(crate) normals: SmallVec<[Vec3; 4]>,
    pub(crate) points: SmallVec<[Vec3; 4]>,
    pub(crate) surfaces: SmallVec<[Surface; 4]>,
}

pub(crate) struct ConvexShapeAxisDetails {
    pub(crate) penetration_depth: Vec3,
}

impl ConvexShapeAxisDetails {
    fn to_opposite(self) -> Self {
        Self {
            penetration_depth: -1. * self.penetration_depth,
        }
    }

    fn is_more_accurate_than(&self, other: &Option<Self>) -> bool {
        other
            .as_ref()
            .map(|o| self.penetration_depth.magnitude() < o.penetration_depth.magnitude())
            .unwrap_or(true)
    }
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
        for &axis in &ref_properties.normals {
            let collision = collision(axis, &ref_properties.points, &other_properties.points)?;
            if collision.is_more_accurate_than(&best_collision) {
                best_collision = Some(collision);
            }
        }
        for &axis in &other_properties.normals {
            let collision =
                collision(axis, &other_properties.points, &ref_properties.points)?.to_opposite();
            if collision.is_more_accurate_than(&best_collision) {
                best_collision = Some(collision);
            }
        }
        best_collision.map(|c| CollisionDetails {
            penetration_depth: c.penetration_depth,
            contact_centroid: calculate_contact_centroid(&ref_properties, &other_properties),
        })
    }
}

fn collision(
    axis: Vec3,
    ref_points: &[Vec3],
    other_points: &[Vec3],
) -> Option<ConvexShapeAxisDetails> {
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

fn partial_collision(axis: Vec3, ref_factor: f32, other_factor: f32) -> ConvexShapeAxisDetails {
    ConvexShapeAxisDetails {
        penetration_depth: axis * (other_factor - ref_factor),
    }
}

fn full_collision(
    axis: Vec3,
    inner_factor1: f32,
    inner_factor2: f32,
    outer_factor1: f32,
    outer_factor2: f32,
) -> ConvexShapeAxisDetails {
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
    ConvexShapeAxisDetails {
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

fn calculate_contact_centroid(
    ref_properties: &ConvexShapeProperties,
    other_properties: &ConvexShapeProperties,
) -> Vec3 {
    let mut contact_centroid = Contact::default();
    for ref_surface in &ref_properties.surfaces {
        for other_surface in &other_properties.surfaces {
            match ref_surface {
                Surface::Segment(p1, p2) => match other_surface {
                    Surface::Segment(p3, p4) => {
                        let intersection = segment_segment_2d::intersection(
                            (p1.xy(), p2.xy()),
                            (p3.xy(), p4.xy()),
                        );
                        match intersection {
                            Segments2DIntersection::Point(p) => {
                                contact_centroid.add_intersection(p.with_z(0.));
                            }
                            Segments2DIntersection::Segment(p1, p2) => {
                                contact_centroid.add_intersection(p1.with_z(0.));
                                contact_centroid.add_intersection(p2.with_z(0.));
                            }
                            Segments2DIntersection::None => {}
                        }
                    }
                    Surface::Triangle(_, _, _) => {
                        panic!("internal error: try to find intersection between 2D and 3D shape");
                    }
                },
                Surface::Triangle(_, _, _) => match other_surface {
                    Surface::Segment(_, _) => {
                        panic!("internal error: try to find intersection between 2D and 3D shape");
                    }
                    Surface::Triangle(_, _, _) => {
                        todo!("implement 3D triangle-triangle collision")
                    }
                },
            }
        }
    }
    contact_centroid.centroid().unwrap_or_else(|| {
        if ref_properties.size.magnitude() > other_properties.size.magnitude() {
            other_properties.position
        } else {
            ref_properties.position
        }
    })
}

#[derive(Default)]
struct Contact {
    centroid: Vec3,
    contact_count: f32,
}

impl Contact {
    fn centroid(&self) -> Option<Vec3> {
        (self.contact_count > 0.5).then(|| self.centroid)
    }

    fn add_intersection(&mut self, point: Vec3) {
        self.centroid = if self.contact_count < 0.5 {
            point
        } else {
            (self.centroid * self.contact_count + point) / (self.contact_count + 1.)
        };
        self.contact_count += 1.;
    }
}
