use crate::{Color, TextureRef};
use modor_internal::dyn_key::DynKey;

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
    /// This color will be applied only if there is no texture or if the texture is not loaded.
    ///
    /// Some optimizations are perform on shapes with alpha component equal to zero.
    pub color: Color,
    /// Z-coordinate of the mesh used to define display order, where smallest Z-coordinates are
    /// displayed first.
    pub z: f32,
    /// Color applied to the attached texture.
    ///
    /// The color of each pixel of the texture will be multiplied component-wise by this color.<br>
    /// This color will be applied only if there is an attached texture that is already loaded.
    pub texture_color: Color,
    pub(crate) texture_key: Option<DynKey>,
    pub(crate) shape: Shape,
}

impl Mesh2D {
    /// Creates a new white rectangle.
    ///
    /// The rectangle size is driven by the [`Transform2D`](modor_physics::Transform2D) size along
    /// X-axis and Y-axis.
    #[must_use]
    pub const fn rectangle() -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            shape: Shape::Rectangle,
            texture_color: Color::WHITE,
            texture_key: None,
        }
    }

    /// Creates a new white ellipse.
    ///
    /// The ellipse major and minor radii are driven by the
    /// [`Transform2D`](modor_physics::Transform2D) size along X-axis and Y-axis.
    #[must_use]
    pub const fn ellipse() -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            shape: Shape::Ellipse,
            texture_color: Color::WHITE,
            texture_key: None,
        }
    }

    /// Returns the mesh with a different `color`.
    ///
    /// Default value is `Color::WHITE`.
    #[must_use]
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the mesh with a different `z`.
    ///
    /// Default value is `0.0`.
    #[must_use]
    pub const fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Returns the mesh with an attached texture with label `texture_label`.
    ///
    /// There is no attached texture by default.
    #[must_use]
    pub fn with_texture(mut self, texture_ref: impl TextureRef) -> Self {
        self.texture_key = Some(DynKey::new(texture_ref));
        self
    }

    /// Returns the mesh with a different texture `texture_color`.
    ///
    /// Default value is `Color::WHITE`.
    #[must_use]
    pub const fn with_texture_color(mut self, texture_color: Color) -> Self {
        self.texture_color = texture_color;
        self
    }

    /// Attach a new texture.
    pub fn attach_texture(&mut self, texture_ref: impl TextureRef) {
        self.texture_key = Some(DynKey::new(texture_ref));
    }

    /// Detach the current texture if any is attached.
    pub fn detach_texture(&mut self) {
        self.texture_key = None;
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Shape {
    Rectangle,
    Ellipse,
}
