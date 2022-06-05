use modor_math::Vector2D;

pub(crate) const DEFAULT_INPUT_STATE: InputState = InputState {
    is_pressed: false,
    is_just_pressed: false,
    is_just_released: false,
};

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

/// The delta of a movable input.
///
/// For X-axis, right corresponds to positive coordinate.<br>
/// For Y-axis, up corresponds to positive coordinate.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Default, Debug, Clone, Copy, Add, Sub, AddAssign, SubAssign)]
pub struct InputDelta {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
}

impl InputDelta {
    /// Creates a new input delta.
    pub fn xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Vector2D for InputDelta {
    fn create(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn components(self) -> (f32, f32) {
        (self.x, self.y)
    }
}
