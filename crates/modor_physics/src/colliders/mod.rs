pub(crate) mod convex_shape;
pub(crate) mod rectangle;
pub(crate) mod utils;

use crate::colliders::rectangle::RectangleCollider;
use crate::entities::collisions::CollisionGroupRelationship;
use crate::{Collider, ColliderShape, Transform};
use modor_math::Vec3;

pub(crate) enum ShapeCollider {
    Rectangle(RectangleCollider),
    Circle(RectangleCollider),
}

impl ShapeCollider {
    pub(crate) fn new(
        collider: &Collider,
        transform: &Transform,
        relationship: &CollisionGroupRelationship,
    ) -> Self {
        match collider.shape {
            ColliderShape::Rectangle => {
                Self::Rectangle(RectangleCollider::new(transform, relationship))
            }
            ColliderShape::Circle => Self::Circle(RectangleCollider::new(transform, relationship)),
        }
    }

    pub(crate) fn check_collision(&self, other: &ShapeCollider) -> Option<CollisionDetails> {
        match self {
            ShapeCollider::Rectangle(collider) => collider.check_collision(other),
            ShapeCollider::Circle(collider) => collider.check_collision(other),
        }
    }
}

trait CollisionCheck {
    fn check_collision(&self, other: &ShapeCollider) -> Option<CollisionDetails>;
}

// TODO: add also collision position
pub(crate) struct CollisionDetails {
    pub(crate) penetration_depth: Vec3,
}

impl CollisionDetails {
    fn is_more_accurate_than(&self, other: &Option<Self>) -> bool {
        other
            .as_ref()
            .map(|o| self.penetration_depth.magnitude() < o.penetration_depth.magnitude())
            .unwrap_or(true)
    }

    fn to_opposite(&self) -> Self {
        Self {
            penetration_depth: self.penetration_depth * -1.,
        }
    }
}
