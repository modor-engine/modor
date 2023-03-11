use crate::Color;
use std::ops::{Deref, DerefMut};

/// The background color of the rendering.
///
/// If no background color is defined, the background is black.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// let app = App::new().with_entity(BackgroundColor::from(Color::RED));
///
/// fn update_color(mut color: SingleMut<'_, BackgroundColor>) {
///     color.r -= 0.001;
/// }
/// ```
#[derive(SingletonComponent, NoSystem)]
pub struct BackgroundColor(Color);

impl From<Color> for BackgroundColor {
    fn from(color: Color) -> Self {
        Self(color)
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
