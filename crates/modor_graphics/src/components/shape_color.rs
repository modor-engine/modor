use crate::Color;
use std::ops::{Deref, DerefMut};

/// The color of an entity.
///
/// An entity with this component will be rendered with the specified color.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Position`](modor_physics::Position), [`Size`](modor_physics::Size)
/// - **Optional components**: [`Shape`](modor_physics::Shape)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, EntityBuilder, Built};
/// # use modor_graphics::{Color, ShapeColor};
/// # use modor_physics::{Position, Size, Shape};
/// #
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Position::xy(-0.25, 0.25))
///             .with(Size::xy(0.5, 0.5))
///             .with(Shape::Rectangle2D)
///             .with(ShapeColor(Color::rgba(1., 0.25, 0.75, 0.5)))
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug)]
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
