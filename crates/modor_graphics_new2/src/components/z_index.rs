#[must_use]
#[derive(Component, NoSystem, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ZIndex2D(u16);

impl ZIndex2D {
    // offset should be between 0. and 0.5
    pub(crate) fn to_f32(&self, offset: f32) -> f32 {
        (f32::from(self.0) + 0.5 + offset) / (f32::from(u16::MAX) + 1.)
    }
}

impl From<u16> for ZIndex2D {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl From<ZIndex2D> for u16 {
    fn from(index: ZIndex2D) -> Self {
        index.0
    }
}
