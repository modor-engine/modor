/// The state of a pressable input.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Default, Debug, Clone, Copy)]
#[non_exhaustive]
pub struct InputState {
    is_pressed: bool,
    is_just_pressed: bool,
    is_just_released: bool,
}

impl InputState {
    pub(crate) const DEFAULT: Self = Self::new();

    /// Creates a released input state.
    pub const fn new() -> Self {
        Self {
            is_pressed: false,
            is_just_pressed: false,
            is_just_released: false,
        }
    }

    /// Returns whether the input is pressed.
    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns whether the input has just been pressed.
    pub const fn is_just_pressed(&self) -> bool {
        self.is_just_pressed
    }

    /// Returns whether the input has just been released.
    pub const fn is_just_released(&self) -> bool {
        self.is_just_released
    }

    /// Presses the input.
    pub fn press(&mut self) {
        self.is_pressed = true;
        self.is_just_pressed = true;
    }

    /// Releases the input.
    pub fn release(&mut self) {
        self.is_pressed = false;
        self.is_just_released = true;
    }

    pub(crate) fn refresh(&mut self) {
        self.is_just_pressed = false;
        self.is_just_released = false;
    }
}
