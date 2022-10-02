#[derive(Clone, Copy)]
pub(crate) struct UserData(u128);

impl UserData {
    pub(crate) fn new(entity_id: usize) -> Self {
        Self(0).with_entity_id(entity_id)
    }

    pub(crate) fn with_entity_id(self, entity_id: usize) -> Self {
        Self(entity_id as u128 * 10 + if self.has_deletion_flag() { 1 } else { 0 })
    }

    pub(crate) fn with_deletion_flag(self, has_deletion_flag: bool) -> Self {
        Self(self.entity_id() as u128 * 10 + if has_deletion_flag { 1 } else { 0 })
    }

    #[allow(clippy::integer_division)]
    pub(crate) fn entity_id(self) -> usize {
        (self.0 / 10) as usize
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
        assert!(!user_data.has_deletion_flag());
        let user_data = UserData::new(0).with_entity_id(15).with_deletion_flag(true);
        assert_eq!(user_data.entity_id(), 15);
        assert!(user_data.has_deletion_flag());
        let user_data = user_data.with_entity_id(30);
        assert_eq!(user_data.entity_id(), 30);
        assert!(user_data.has_deletion_flag());
        let user_data = user_data.with_deletion_flag(false);
        assert_eq!(user_data.entity_id(), 30);
        assert!(!user_data.has_deletion_flag());
    }
}
