use crate::colliders::convex_shape::{ConvexShape, ConvexShapeProperties, Surface};
use crate::colliders::{CollisionCheck, CollisionDetails, ShapeCollider};
use crate::entities::collisions::CollisionGroupRelationship;
use crate::Transform;
use modor_math::{Mat4, Vec3};
use smallvec::smallvec;

pub(crate) struct RectangleCollider {
    position: Vec3,
    size: Vec3,
    matrix: Mat4,
    ignore_z: bool,
}

impl RectangleCollider {
    pub(crate) fn new(transform: &Transform, relationship: &CollisionGroupRelationship) -> Self {
        Self {
            position: transform.position,
            size: transform.size,
            matrix: transform.create_matrix(),
            ignore_z: relationship.ignore_z,
        }
    }
}

impl ConvexShape for RectangleCollider {
    fn properties(&self) -> ConvexShapeProperties {
        let point1 = self.matrix * Vec3::from_xy(-0.5, 0.5);
        let point2 = self.matrix * Vec3::from_xy(-0.5, -0.5);
        let point3 = self.matrix * Vec3::from_xy(0.5, -0.5);
        let point4 = self.matrix * Vec3::from_xy(0.5, 0.5);
        let x_axis = point3 - point2;
        let y_axis = point1 - point2;
        ConvexShapeProperties {
            position: self.position,
            size: self.size,
            normals: if self.ignore_z {
                smallvec![x_axis, y_axis]
            } else {
                smallvec![x_axis, y_axis, x_axis.cross(y_axis)]
            },
            points: smallvec![point1, point2, point3, point4],
            surfaces: if self.ignore_z {
                smallvec![
                    Surface::Segment(point1, point2),
                    Surface::Segment(point2, point3),
                    Surface::Segment(point3, point4),
                    Surface::Segment(point4, point1),
                ]
            } else {
                smallvec![
                    Surface::Triangle(point1, point2, point3),
                    Surface::Triangle(point1, point4, point3)
                ]
            },
        }
    }
}

impl CollisionCheck for RectangleCollider {
    fn check_collision(&self, other: &ShapeCollider) -> Option<CollisionDetails> {
        match other {
            ShapeCollider::Rectangle(other) => ConvexShape::check_collision(self, other),
            ShapeCollider::Circle(other) => ConvexShape::check_collision(self, other),
        }
    }
}
