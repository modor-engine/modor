use crate::Color;
use modor::{Built, EntityBuilder};

pub struct BackgroundColor(pub Color);

#[singleton]
impl BackgroundColor {
    pub fn build(color: Color) -> impl Built<Self> {
        EntityBuilder::new(Self(color))
    }
}
