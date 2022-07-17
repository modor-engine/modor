use crate::entities::collisions::GroupIdx;
use crate::CollisionGroup;
use modor_math::Vec3;

pub struct Collider {
    pub(crate) shape: ColliderShape,
    pub(crate) group_idx: GroupIdx,
    pub(crate) collisions: Vec<Collision>,
}

impl Collider {
    pub fn rectangle(group: impl CollisionGroup) -> Self {
        Self {
            shape: ColliderShape::Rectangle,
            group_idx: group.index().into(),
            collisions: vec![],
        }
    }

    pub fn circle<G>(group: impl CollisionGroup) -> Self {
        Self {
            shape: ColliderShape::Circle,
            group_idx: group.index().into(),
            collisions: vec![],
        }
    }

    pub fn collisions(&self) -> &[Collision] {
        &self.collisions
    }
}

#[derive(Clone, Debug)]
pub struct Collision {
    pub(crate) entity_id: usize,
    pub(crate) other_entity_id: usize,
    pub(crate) other_entity_group_idx: GroupIdx,
    pub(crate) normal: Vec3,
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

    pub fn normal(&self) -> Vec3 {
        self.normal
    }
}

pub(crate) enum ColliderShape {
    Rectangle,
    Circle,
}
