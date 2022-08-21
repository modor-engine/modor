use crate::colliders::circle_2d::Circle2DCollider;
use crate::colliders::convex_2d::Convex2DCollider;
use crate::collisions::{circle_circle_2d, convex_convex_2d, CollisionDetails};
use crate::entities::collisions::GroupIdx;
use crate::{CollisionGroup, Transform};
use modor_math::Vec3;

pub struct Collider {
    pub(crate) shape: ColliderShape,
    pub(crate) simplified_shape: ColliderSimplifiedShape,
    pub(crate) group_idx: GroupIdx,
    pub(crate) collisions: Vec<Collision>,
}

// TODO: add 3d shapes
impl Collider {
    pub fn rectangle_2d(group: impl CollisionGroup) -> Self {
        Self {
            shape: ColliderShape::Rectangle2D(Convex2DCollider::default()),
            simplified_shape: ColliderSimplifiedShape::Circle2D(Circle2DCollider::default()),
            group_idx: group.index().into(),
            collisions: vec![],
        }
    }

    pub fn circle_2d(group: impl CollisionGroup) -> Self {
        Self {
            shape: ColliderShape::Circle2D(Circle2DCollider::default()),
            simplified_shape: ColliderSimplifiedShape::Circle2D(Circle2DCollider::default()),
            group_idx: group.index().into(),
            collisions: vec![],
        }
    }

    pub fn collisions(&self) -> &[Collision] {
        &self.collisions
    }

    pub(crate) fn update(&mut self, transform: &Transform) {
        match &mut self.shape {
            ColliderShape::Rectangle2D(shape) => shape.update(transform),
            ColliderShape::Circle2D(shape) => shape.update_inside_collider(transform),
        }
        match &mut self.simplified_shape {
            ColliderSimplifiedShape::Circle2D(shape) => shape.update_outside_collider(transform),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Collision {
    pub(crate) entity_id: usize,
    pub(crate) other_entity_id: usize,
    pub(crate) other_entity_group_idx: GroupIdx,
    pub(crate) penetration: Vec3,
    pub(crate) contact_centroid: Vec3,
}

impl Collision {
    pub fn other_entity_id(&self) -> usize {
        self.other_entity_id
    }

    pub fn is_other_entity_from_group<G>(&self, group: G) -> bool
    where
        G: CollisionGroup,
    {
        self.other_entity_group_idx == group.index().into()
    }

    pub fn penetration(&self) -> Vec3 {
        self.penetration
    }

    pub fn contact_centroid(&self) -> Vec3 {
        self.contact_centroid
    }
}

pub(crate) enum ColliderShape {
    Rectangle2D(Convex2DCollider),
    Circle2D(Circle2DCollider),
}

impl ColliderShape {
    pub(crate) fn check_collision(&self, other: &Self) -> Option<CollisionDetails> {
        match self {
            Self::Rectangle2D(shape1) => match other {
                Self::Rectangle2D(shape2) => convex_convex_2d::collision(shape1, shape2),
                Self::Circle2D(shape2) => todo!(),
            },
            Self::Circle2D(shape1) => match other {
                Self::Rectangle2D(shape2) => todo!(),
                Self::Circle2D(shape2) => circle_circle_2d::collision(shape1, shape2),
            },
        }
    }
}

pub(crate) enum ColliderSimplifiedShape {
    Circle2D(Circle2DCollider),
}

impl ColliderSimplifiedShape {
    pub(crate) fn is_colliding_with(&self, other: &Self) -> bool {
        match self {
            Self::Circle2D(shape1) => match other {
                Self::Circle2D(shape2) => circle_circle_2d::are_colliding(shape1, shape2),
            },
        }
    }
}
