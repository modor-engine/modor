use crate::storages::resources::fonts::{DefaultFontKey, FontKey};
use crate::storages::texts::TextIdx;
use crate::{Color, FontRef};

/// The properties of an entity rendered as a 2D text.
///
/// In order to optimize the rendering, the text is cached in an internal texture.<br>
/// This texture is updated only when the text `string`, the font or the `font_height` are modified.
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
/// # use modor_graphics::{Text2D, Color, Alignment, TextSize};
/// #
/// struct Text;
///
/// #[entity]
/// impl Text {
///     fn build(position: Vec2, size: Vec2, string: &str, color: Color) -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(
///                 Transform2D::new()
///                     .with_position(position)
///                     .with_size(size)
///             )
///             .with(
///                 Text2D::new(100., string)
///                     .with_color(color)
///                     .with_z(2.)
///                     .with_size(TextSize::LineHeight(size.y / 5.)) // display on 5 lines
///                     .with_alignment(Alignment::TopLeft)
///             )
///     }
/// }
/// ```
///
/// See also [`Font`](crate::Font) for a font attachment example.
#[derive(Debug, Clone, Component)]
pub struct Text2D {
    /// The string to render.
    pub string: String,
    /// The font height in pixels used to create the internal texture.
    pub font_height: f32,
    /// The text size.
    pub size: TextSize,
    /// The text alignment.
    pub alignment: Alignment,
    /// The text color.
    pub color: Color,
    /// Z-coordinate of the text used to define display order, where smallest Z-coordinates are
    /// displayed first.
    pub z: f32,
    pub(crate) font_key: FontKey,
    pub(crate) text_idx: Option<TextIdx>,
}

impl Text2D {
    /// Creates a new text.
    pub fn new(font_height: f32, string: impl Into<String>) -> Self {
        Self {
            string: string.into(),
            font_height,
            size: TextSize::Auto,
            alignment: Alignment::Center,
            font_key: FontKey::new(DefaultFontKey),
            text_idx: None,
            color: Color::WHITE,
            z: 0.,
        }
    }

    /// Returns the text with a different font.
    ///
    /// Default font is [Roboto Regular](https://fonts.google.com/specimen/Roboto).
    pub fn with_font(mut self, font_ref: impl FontRef) -> Self {
        self.font_key = FontKey::new(font_ref);
        self
    }

    /// Returns the text with a different size.
    ///
    /// Default value is `TextSize::AUTO`.
    pub fn with_size(mut self, size: TextSize) -> Self {
        self.size = size;
        self
    }

    /// Returns the text with a different alignment.
    ///
    /// Default value is `Alignment::Center`.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Returns the text with a different color.
    ///
    /// Default value is `Color::WHITE`.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the text with a different `z`.
    ///
    /// Default value is `0.0`.
    pub const fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Sets the font.
    pub fn set_font(&mut self, font_ref: impl FontRef) {
        self.font_key = FontKey::new(font_ref);
    }

    /// Sets the font to default font.
    ///
    /// Default font is [Roboto Regular](https://fonts.google.com/specimen/Roboto).
    pub fn use_default_font(&mut self) {
        self.font_key = FontKey::new(DefaultFontKey);
    }
}

/// The size of a text to render.
///
/// # Examples
///
/// See [`Text2D`](crate::Text2D).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextSize {
    /// Text fits in the associated [`Transform2D`](modor_physics::Transform2D).
    Auto,
    /// Text lines have a fixed height in world units.
    LineHeight(f32),
}

/// The alignment of a text.
///
/// # Examples
///
/// See [`Text2D`](crate::Text2D).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    /// Top-left alignment.
    TopLeft,
    /// Top alignment.
    Top,
    /// Top-right alignment.
    TopRight,
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
    /// Bottom-left alignment.
    BottomLeft,
    /// Bottom alignment.
    Bottom,
    /// Bottom-right alignment.
    BottomRight,
}
