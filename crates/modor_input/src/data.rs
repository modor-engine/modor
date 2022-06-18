use modor_math::{Point2D, Vec2D};
use std::ops::{Deref, DerefMut};

/// The state of a pressable input.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Default, Clone, Copy)]
pub struct InputState {
    is_pressed: bool,
    is_just_pressed: bool,
    is_just_released: bool,
}

impl InputState {
    /// Returns whether the input is pressed.
    pub fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns whether has just been pressed.
    pub fn is_just_pressed(&self) -> bool {
        self.is_just_pressed
    }

    /// Returns whether has just been released.
    pub fn is_just_released(&self) -> bool {
        self.is_just_released
    }

    pub(crate) fn refresh(&mut self) {
        self.is_just_pressed = false;
        self.is_just_released = false;
    }

    pub(crate) fn press(&mut self) {
        self.is_pressed = true;
        self.is_just_pressed = true;
    }

    pub(crate) fn release(&mut self) {
        self.is_pressed = false;
        self.is_just_released = true;
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InputUnit;

#[derive(Default, Debug, Clone, Copy)]
pub struct Pixels;

// TODO: delete following types ?

/// The delta of a movable input.
///
/// For X-axis, right corresponds to positive coordinate.<br>
/// For Y-axis, up corresponds to positive coordinate.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Default, Debug, Clone, Copy)]
pub struct InputDelta(Vec2D<InputUnit>);

impl InputDelta {
    /// Creates a new input delta.
    pub fn xy(x: f32, y: f32) -> Self {
        Self(Vec2D::xy(x, y))
    }
}

impl Deref for InputDelta {
    type Target = Vec2D<InputUnit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InputDelta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A position in pixels from the top-left corner of the app window.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Default, Clone, Copy, Debug)]
pub struct WindowPosition(Point2D<Pixels>);

impl WindowPosition {
    /// Creates a new position.
    pub fn xy(x: f32, y: f32) -> Self {
        Self(Point2D::xy(x, y))
    }
}

impl Deref for WindowPosition {
    type Target = Point2D<Pixels>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WindowPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
