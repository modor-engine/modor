use crate::storages::resources::textures::TextureKey;
use crate::{Color, TextureRef};
use modor_math::Vec2;

/// The properties of an entity rendered as a 2D mesh.
///
/// The entity will be rendered only if it also has a [`Transform2D`](modor_physics::Transform2D)
/// component.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// # use modor_graphics::*;
/// #
/// fn build_rectangle(position: Vec2, size: Vec2, angle: f32, color: Color) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(
///              Transform2D::new()
///                  .with_position(position)
///                  .with_size(size)
///                  .with_rotation(angle)
///          )
///          .with(
///              Mesh2D::rectangle()
///                  .with_color(color)
///                  .with_z(2.)
///          )
/// }
/// ```
///
/// See also [`Texture`](crate::Texture) for a texture attachment example.
#[derive(Clone, Debug, Component, NoSystem)]
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
    /// Configuration of the texture part to use.
    pub texture_part: TexturePart,
    pub(crate) texture_key: Option<TextureKey>,
    pub(crate) shape: Shape,
}

impl Mesh2D {
    /// Creates a new white rectangle.
    ///
    /// The rectangle size is driven by the [`Transform2D`](modor_physics::Transform2D) size along
    /// X-axis and Y-axis.
    pub const fn rectangle() -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            shape: Shape::Rectangle,
            texture_color: Color::WHITE,
            texture_key: None,
            texture_part: TexturePart::DEFAULT,
        }
    }

    /// Creates a new white ellipse.
    ///
    /// The ellipse major and minor radii are driven by the
    /// [`Transform2D`](modor_physics::Transform2D) size along X-axis and Y-axis.
    pub const fn ellipse() -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            shape: Shape::Ellipse,
            texture_color: Color::WHITE,
            texture_key: None,
            texture_part: TexturePart::DEFAULT,
        }
    }

    /// Returns the mesh with a different `color`.
    ///
    /// Default value is `Color::WHITE`.
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the mesh with a different `z`.
    ///
    /// Default value is `0.0`.
    pub const fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Returns the mesh with an attached texture with label `texture_label`.
    ///
    /// There is no attached texture by default.
    pub fn with_texture(mut self, texture_ref: impl TextureRef) -> Self {
        self.texture_key = Some(TextureKey::new(texture_ref));
        self
    }

    /// Returns the mesh with an attached texture with label `texture_label`.
    ///
    /// By default, the whole texture is used.
    pub fn with_texture_part(mut self, texture_part: TexturePart) -> Self {
        self.texture_part = texture_part;
        self
    }

    /// Returns the mesh with a different texture `texture_color`.
    ///
    /// Default value is `Color::WHITE`.
    pub const fn with_texture_color(mut self, texture_color: Color) -> Self {
        self.texture_color = texture_color;
        self
    }

    /// Attach a new texture.
    pub fn attach_texture(&mut self, texture_ref: impl TextureRef) {
        self.texture_key = Some(TextureKey::new(texture_ref));
    }

    /// Detach the current texture if any is attached.
    pub fn detach_texture(&mut self) {
        self.texture_key = None;
    }
}

/// The part of a texture to apply.
///
/// This can for example be used to apply a part of a spritesheet to a mesh.
///
/// # Examples
///
/// See [`Texture`](crate::Texture)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TexturePart {
    /// The top-left position of the texture part in the texture.
    ///
    /// `Vec2::ZERO` corresponds to top-left corner of the texture.<br>
    /// `Vec2::ONE` corresponds to bottom-right corner of the texture.
    pub position: Vec2,
    /// The size of the texture part in the texture.
    ///
    /// The texture has a size of `Vec2::ONE`.
    pub size: Vec2,
}

impl TexturePart {
    const DEFAULT: Self = Self {
        position: Vec2::ZERO,
        size: Vec2::ONE,
    };

    /// Returns the texture part with a different position.
    ///
    /// Default value is `Vec2::ZERO`.
    pub fn with_position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    /// Returns the texture part with a different size.
    ///
    /// Default value is `Vec2::ONE`.
    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
}

impl Default for TexturePart {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Shape {
    Rectangle,
    Ellipse,
}
