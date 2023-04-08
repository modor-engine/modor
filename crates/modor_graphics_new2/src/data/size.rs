use modor_math::Vec2;
use std::num::NonZeroU32;
use winit::dpi::{PhysicalSize, Size as WinitSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub const ONE: Self = Self::new(1, 1);

    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl From<Size> for WinitSize {
    fn from(size: Size) -> Self {
        Self::Physical(PhysicalSize::new(size.width, size.height))
    }
}

impl From<Size> for Vec2 {
    #[allow(clippy::cast_precision_loss)]
    fn from(size: Size) -> Self {
        Self::new(size.width as f32, size.height as f32)
    }
}

// This type is useful to avoid `Surface` panic because of width or height = 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NonZeroSize {
    pub(crate) width: NonZeroU32,
    pub(crate) height: NonZeroU32,
}

impl From<Size> for NonZeroSize {
    fn from(size: Size) -> Self {
        Self {
            width: NonZeroU32::new(size.width.max(1)).unwrap_or_else(|| unreachable!()),
            height: NonZeroU32::new(size.height.max(1)).unwrap_or_else(|| unreachable!()),
        }
    }
}

impl From<NonZeroSize> for Size {
    fn from(size: NonZeroSize) -> Self {
        Self {
            width: size.width.into(),
            height: size.height.into(),
        }
    }
}

impl From<NonZeroSize> for Vec2 {
    #[allow(clippy::cast_precision_loss)]
    fn from(size: NonZeroSize) -> Self {
        Self::new(u32::from(size.width) as f32, u32::from(size.height) as f32)
    }
}
