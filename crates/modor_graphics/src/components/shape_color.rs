use crate::Color;
use std::ops::{Deref, DerefMut};

// TODO: create Mesh component + Shape should not be a component anymore

/// The color of an entity.
///
/// An entity with this component will be rendered with the specified color.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform`](modor_physics::Transform)
/// - **Optional components**: [`Shape`](crate::Shape)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, EntityBuilder, Built};
/// # use modor_graphics::{Color, ShapeColor, Shape};
/// # use modor_physics::Transform;
/// # use modor_math::Vec3;
/// #
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(
///                 Transform::new()
///                     .with_position(Vec3::xy(-0.25, 0.25))
///                     .with_size(Vec3::xy(0.5, 0.5))
///             )
///             .with(Shape::Rectangle)
///             .with(ShapeColor::from(Color::rgba(1., 0.25, 0.75, 0.5)))
///     }
/// }
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ShapeColor(Color);

impl From<Color> for ShapeColor {
    fn from(color: Color) -> Self {
        Self(color)
    }
}

impl From<ShapeColor> for Color {
    fn from(color: ShapeColor) -> Self {
        color.0
    }
}

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
