use crate::Color;
use std::marker::PhantomData;

/// The properties of a rendered entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform`](modor_physics::Transform)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, Built, EntityBuilder};
/// # use modor_math::{Vec3, Quat};
/// # use modor_physics::Transform;
/// # use modor_graphics::{Mesh, Color};
/// #
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build(position: Vec3, size: Vec3, angle: f32, color: Color) -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(
///                 Transform::new()
///                     .with_position(position)
///                     .with_size(size)
///                     .with_rotation(Quat::from_z(angle))
///             )
///             .with(Mesh::rectangle().with_color(color))
///     }
/// }
/// ```
pub struct Mesh {
    /// Color of the entity.
    pub color: Color,
    pub(crate) shape: Shape,
    phantom: PhantomData<()>,
}

impl Mesh {
    /// Creates a new white rectangle.
    ///
    /// The rectangle size is driven by the [`Transform`](modor_physics::Transform) size along
    /// X-axis and Y-axis.
    pub const fn rectangle() -> Self {
        Self {
            color: Color::WHITE,
            shape: Shape::Rectangle,
            phantom: PhantomData,
        }
    }

    /// Creates a new white ellipse.
    ///
    /// The ellipse major and minor radii are driven by the [`Transform`](modor_physics::Transform)
    /// size along X-axis and Y-axis.
    pub const fn ellipse() -> Self {
        Self {
            color: Color::WHITE,
            shape: Shape::Ellipse,
            phantom: PhantomData,
        }
    }

    /// Returns the mesh with a different `color`.
    #[must_use]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

pub(crate) enum Shape {
    Rectangle,
    Ellipse,
}
