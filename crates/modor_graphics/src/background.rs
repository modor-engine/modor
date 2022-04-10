use crate::Color;
use modor::{Built, EntityBuilder};
use std::ops::{Deref, DerefMut};

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
