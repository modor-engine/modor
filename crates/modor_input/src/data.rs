/// The state of a pressable input.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Default, Clone, Copy)]
#[non_exhaustive]
pub struct InputState {
    /// Whether the input is pressed.
    pub is_pressed: bool,
    /// Whether the input has just been pressed.
    pub is_just_pressed: bool,
    /// Whether the input has just been released.
    pub is_just_released: bool,
}

impl InputState {
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
