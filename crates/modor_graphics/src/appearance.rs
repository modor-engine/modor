#[derive(Clone, Debug)]
pub struct ShapeColor(pub Color);

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
    // TODO: add more colors (https://www.rapidtables.com/web/color/RGB_Color.html)
    pub const BLACK: Color = Color::rgb(0., 0., 0.);
    pub const WHITE: Color = Color::rgb(1., 1., 1.);
    pub const RED: Color = Color::rgb(1., 0., 0.);
    pub const GREEN: Color = Color::rgb(0., 1., 0.);
    pub const BLUE: Color = Color::rgb(0., 0., 1.);
    pub const INVISIBLE: Color = Color::rgba(0., 0., 0., 0.);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
    }
}
