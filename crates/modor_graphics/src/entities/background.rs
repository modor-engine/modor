use crate::Color;
use modor::{Built, EntityBuilder};
use std::ops::{Deref, DerefMut};

/// The background color of the rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Default if missing**: `BackgroundColor::build(Color::BLACK)`
///
/// # Examples
///
/// ```rust
/// # use modor::{App, SingleMut};
/// # use modor_graphics::{BackgroundColor, Color};
/// #
/// let app = App::new().with_entity(BackgroundColor::build(Color::RED));
///
/// fn update_color(mut color: SingleMut<'_, BackgroundColor>) {
///     color.r -= 0.001;
/// }
/// ```
pub struct BackgroundColor(Color);

#[singleton]
impl BackgroundColor {
    /// Builds the entity.
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
