#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl SurfaceSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
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
    pub const BLACK: Color = Color::rgb(0., 0., 0.);
    pub const SILVER: Color = Color::rgb(0.75, 0.75, 0.75);
    pub const GRAY: Color = Color::rgb(0.5, 0.5, 0.5);
    pub const DARK_GRAY: Color = Color::rgb(0.25, 0.25, 0.25);
    pub const WHITE: Color = Color::rgb(1., 1., 1.);
    pub const RED: Color = Color::rgb(1., 0., 0.);
    pub const GREEN: Color = Color::rgb(0., 1., 0.);
    pub const BLUE: Color = Color::rgb(0., 0., 1.);
    pub const YELLOW: Color = Color::rgb(1., 1., 0.);
    pub const CYAN: Color = Color::rgb(0., 1., 1.);
    pub const MAGENTA: Color = Color::rgb(1., 0., 1.);
    pub const MAROON: Color = Color::rgb(0.5, 0., 0.);
    pub const DARK_GREEN: Color = Color::rgb(0., 0.5, 0.);
    pub const NAVY: Color = Color::rgb(0., 0., 0.5);
    pub const OLIVE: Color = Color::rgb(0.5, 0.5, 0.);
    pub const TEAL: Color = Color::rgb(0., 0.5, 0.5);
    pub const PURPLE: Color = Color::rgb(0.5, 0., 0.5);
    pub const INVISIBLE: Color = Color::rgba(0., 0., 0., 0.);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
}
