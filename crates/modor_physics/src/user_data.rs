#[derive(Debug, Clone, Copy)]
pub(crate) struct ColliderUserData(u128);

impl ColliderUserData {
    #[allow(clippy::cast_lossless)]
    pub(crate) fn new(body_index: usize, group_index: usize) -> Self {
        Self(group_index as u128 | ((body_index as u128) << 64))
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn body_index(self) -> usize {
        (self.0 >> 64) as usize
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn group_index(self) -> usize {
        ((self.0 << 64) >> 64) as usize
    }
}

impl From<u128> for ColliderUserData {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<ColliderUserData> for u128 {
    fn from(data: ColliderUserData) -> Self {
        data.0
    }
}

#[cfg(test)]
mod user_data_tests {
    use crate::user_data::ColliderUserData;

    #[modor::test]
    fn create() {
        let data = ColliderUserData::new(42, 78);
        assert_eq!(data.body_index(), 42);
        assert_eq!(data.group_index(), 78);
    }
}
