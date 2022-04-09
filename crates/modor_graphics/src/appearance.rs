use modor::{Built, EntityBuilder};
use std::ops::{Deref, DerefMut};

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
    // TODO: add rgb(a)_u8 methods
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

pub struct BackgroundColor(Color);

#[singleton]
impl BackgroundColor {
    pub fn build(color: Color) -> impl Built<Self> {
        EntityBuilder::new(Self(color))
    }
}

impl Deref for BackgroundColor {
    type Target = Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BackgroundColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl SurfaceSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
