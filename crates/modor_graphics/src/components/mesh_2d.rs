use crate::Color;

/// The properties of a rendered entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform2D`](modor_physics::Transform2D)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, Built, EntityBuilder};
/// # use modor_math::Vec2;
/// # use modor_physics::Transform2D;
/// # use modor_graphics::{Mesh2D, Color};
/// #
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build(position: Vec2, size: Vec2, angle: f32, color: Color) -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(
///                 Transform2D::new()
///                     .with_position(position)
///                     .with_size(size)
///                     .with_rotation(angle)
///             )
///             .with(
///                 Mesh2D::rectangle()
///                     .with_color(color)
///                     .with_z(2.)
///             )
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Mesh2D {
    /// Color of the entity.
    ///
    /// Some optimizations are perform on shapes with alpha component equal to zero.
    pub color: Color,
    /// Z-coordinate of the mesh used to define display order, where smallest Z-coordinates are
    /// displayed first.
    pub z: f32,
    pub(crate) shape: Shape,
}

impl Mesh2D {
    /// Creates a new white rectangle.
    ///
    /// The rectangle size is driven by the [`Transform`](modor_physics::Transform) size along
    /// X-axis and Y-axis.
    #[must_use]
    pub const fn rectangle() -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            shape: Shape::Rectangle,
        }
    }

    /// Creates a new white ellipse.
    ///
    /// The ellipse major and minor radii are driven by the [`Transform`](modor_physics::Transform)
    /// size along X-axis and Y-axis.
    #[must_use]
    pub const fn ellipse() -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            shape: Shape::Ellipse,
        }
    }

    /// Returns the mesh with a different `color`.
    #[must_use]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the mesh with a different `z`.
    #[must_use]
    pub const fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Shape {
    Rectangle,
    Ellipse,
}
