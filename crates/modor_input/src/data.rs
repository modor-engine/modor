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
