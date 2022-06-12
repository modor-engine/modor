/// The size in pixels of a surface in which rendering is done.
///
/// # Examples
///
/// See [`GraphicsModule`](crate::GraphicsModule).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SurfaceSize {
    /// Surface width in pixels.
    pub width: u32,
    /// Surface height in pixels.
    pub height: u32,
}

impl SurfaceSize {
    /// Creates a new size.
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

/// A color.
///
/// # Examples
///
/// See [`ShapeColor`](crate::ShapeColor).
#[derive(Clone, Copy, Debug)]
pub struct Color {
    /// Red component between `0.0` and `1.0`.
    pub r: f32,
    /// Green component between `0.0` and `1.0`.
    pub g: f32,
    /// Blue component between `0.0` and `1.0`.
    pub b: f32,
    /// Alpha component between `0.0` and `1.0`.
    pub a: f32,
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        Self {
            r: color.r.into(),
            g: color.g.into(),
            b: color.b.into(),
            a: color.a.into(),
        }
    }
}

impl Color {
    /// <span style="color:black">█</span>
    pub const BLACK: Self = Self::rgb(0., 0., 0.);
    /// <span style="color:#404040">█</span>
    pub const DARK_GRAY: Self = Self::rgb(0.25, 0.25, 0.25);
    /// <span style="color:gray">█</span>
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    /// <span style="color:silver">█</span>
    pub const SILVER: Self = Self::rgb(0.75, 0.75, 0.75);
    /// <span style="color:white">█</span>
    pub const WHITE: Self = Self::rgb(1., 1., 1.);
    /// <span style="color:red">█</span>
    pub const RED: Self = Self::rgb(1., 0., 0.);
    /// <span style="color:lime">█</span>
    pub const GREEN: Self = Self::rgb(0., 1., 0.);
    /// <span style="color:blue">█</span>
    pub const BLUE: Self = Self::rgb(0., 0., 1.);
    /// <span style="color:yellow">█</span>
    pub const YELLOW: Self = Self::rgb(1., 1., 0.);
    /// <span style="color:cyan">█</span>
    pub const CYAN: Self = Self::rgb(0., 1., 1.);
    /// <span style="color:magenta">█</span>
    pub const MAGENTA: Self = Self::rgb(1., 0., 1.);
    /// <span style="color:maroon">█</span>
    pub const MAROON: Self = Self::rgb(0.5, 0., 0.);
    /// <span style="color:green">█</span>
    pub const DARK_GREEN: Self = Self::rgb(0., 0.5, 0.);
    /// <span style="color:navy">█</span>
    pub const NAVY: Self = Self::rgb(0., 0., 0.5);
    /// <span style="color:olive">█</span>
    pub const OLIVE: Self = Self::rgb(0.5, 0.5, 0.);
    /// <span style="color:teal">█</span>
    pub const TEAL: Self = Self::rgb(0., 0.5, 0.5);
    /// <span style="color:purple">█</span>
    pub const PURPLE: Self = Self::rgb(0.5, 0., 0.5);
    /// No color
    pub const INVISIBLE: Self = Self::rgba(0., 0., 0., 0.);

    /// Creates a new translucent color from components between `0.0` and `1.0`.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a new opaque color from components between `0.0` and `1.0`.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
}
