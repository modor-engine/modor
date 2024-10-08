/// A color.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
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

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        Self::rgba(color[0], color[1], color[2], color[3])
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

    /// Returns the color with a different `red` component.
    pub const fn with_red(mut self, red: f32) -> Self {
        self.r = red;
        self
    }

    /// Returns the color with a different `green` component.
    pub const fn with_green(mut self, green: f32) -> Self {
        self.g = green;
        self
    }

    /// Returns the color with a different `blue` component.
    pub const fn with_blue(mut self, blue: f32) -> Self {
        self.b = blue;
        self
    }

    /// Returns the color with a different `alpha` component.
    pub const fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha;
        self
    }
}
