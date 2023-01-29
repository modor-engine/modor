use crate::Color;
use modor::{Built, EntityBuilder};
use std::ops::{Deref, DerefMut};

// TODO: remove these constants
pub(crate) const DEFAULT_TARGET_WIDTH: u32 = 800;
pub(crate) const DEFAULT_TARGET_HEIGHT: u32 = 600;

// TODO: make window size and resolution independent
// as it is modified by the engine in window mode, cannot be modified, so can only be set in windowless mode
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[singleton]
impl Resolution {
    // TODO: cannot be built by user, specified when creating GraphicsModule
    pub fn build(width: u32, height: u32) -> impl Built<Self> {
        EntityBuilder::new(Self { width, height })
    }

    #[allow(clippy::cast_precision_loss)]
    pub(crate) fn size_f32(&self) -> (f32, f32) {
        (self.width as f32, self.height as f32)
    }
}

pub struct BackgroundColor(Color);

#[singleton]
impl BackgroundColor {
    pub fn build(color: Color) -> impl Built<Self> {
        EntityBuilder::new(Self(color))
    }
}

impl Deref for BackgroundColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BackgroundColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
