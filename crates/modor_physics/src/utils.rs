use crate::storages_2d::collision_groups::CollisionGroupIdx;

const ENTITY_ID_MASK: u128 =
    0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111 << 64;
const COLLISION_GROUP_ID_MASK: u128 =
    0b111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111 << 1;
const DELETION_FLAG_MASK: u128 = 0b1;

#[derive(Clone, Copy)]
pub(crate) struct UserData(u128);

impl UserData {
    pub(crate) fn new(entity_id: usize) -> Self {
        Self(0).with_entity_id(entity_id)
    }

    pub(crate) fn with_entity_id(self, entity_id: usize) -> Self {
        Self((self.0 & !ENTITY_ID_MASK) | ((entity_id as u128) << 64))
    }

    pub(crate) fn with_collision_group_idx(self, collision_group_idx: CollisionGroupIdx) -> Self {
        Self((self.0 & !COLLISION_GROUP_ID_MASK) | ((collision_group_idx.0 as u128) << 1))
    }

    pub(crate) fn with_deletion_flag(self, has_deletion_flag: bool) -> Self {
        Self((self.0 & !DELETION_FLAG_MASK) | u128::from(has_deletion_flag))
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn entity_id(self) -> usize {
        ((self.0 & ENTITY_ID_MASK) >> 64) as usize
    }

    pub(crate) fn collision_group_idx(self) -> CollisionGroupIdx {
        CollisionGroupIdx(((self.0 & COLLISION_GROUP_ID_MASK) >> 1) as usize)
    }

    pub(crate) fn has_deletion_flag(self) -> bool {
        self.0 % 2 == 1
    }
}

impl From<u128> for UserData {
    fn from(data: u128) -> Self {
        Self(data)
    }
}

impl From<UserData> for u128 {
    fn from(data: UserData) -> Self {
        data.0
    }
}

#[cfg(test)]
mod user_data_tests {
    use super::*;

    #[test]
    fn create() {
        let user_data = UserData::new(5);
        assert_eq!(user_data.entity_id(), 5);
        assert_eq!(user_data.collision_group_idx(), CollisionGroupIdx(0));
        assert!(!user_data.has_deletion_flag());
        let user_data = UserData::new(0)
            .with_entity_id(15)
            .with_collision_group_idx(CollisionGroupIdx(42))
            .with_deletion_flag(true);
        assert_eq!(user_data.entity_id(), 15);
        assert_eq!(user_data.collision_group_idx(), CollisionGroupIdx(42));
        assert!(user_data.has_deletion_flag());
        let user_data = user_data.with_entity_id(30);
        assert_eq!(user_data.entity_id(), 30);
        assert_eq!(user_data.collision_group_idx(), CollisionGroupIdx(42));
        assert!(user_data.has_deletion_flag());
        let user_data = user_data.with_collision_group_idx(CollisionGroupIdx(12));
        assert_eq!(user_data.entity_id(), 30);
        assert_eq!(user_data.collision_group_idx(), CollisionGroupIdx(12));
        assert!(user_data.has_deletion_flag());
        let user_data = user_data.with_deletion_flag(false);
        assert_eq!(user_data.entity_id(), 30);
        assert_eq!(user_data.collision_group_idx(), CollisionGroupIdx(12));
        assert!(!user_data.has_deletion_flag());
    }
}
