#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Side {
    Left,
    Right,
}

impl Side {
    pub(crate) fn x_sign(self) -> f32 {
        match self {
            Self::Left => -1.,
            Self::Right => 1.,
        }
    }
}
