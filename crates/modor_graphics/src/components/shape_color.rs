use crate::Color;
use std::ops::{Deref, DerefMut};

/// The color of a shape.
///
/// This component makes an entity renderable with a specific color.
///
/// # Modor
///
/// - **Type**: component
/// - **Other required components**: [`Position`](modor_physics::Position)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, EntityBuilder, Built};
/// # use modor_graphics::{Color, ShapeColor};
/// # use modor_physics::{Position, Scale, Shape};
/// #
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Position::xy(-0.25, 0.25))
///             .with(Scale::xy(0.5, 0.5))
///             .with(Shape::Rectangle2D)
///             .with(ShapeColor(Color::rgba(1., 0.25, 0.75, 0.5)))
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ShapeColor(pub Color);

impl Deref for ShapeColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ShapeColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
