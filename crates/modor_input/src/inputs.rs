use crate::{Fingers, Gamepads, Keyboard, Mouse};
use modor::{Node, RootNode, Visit};

/// The state of the inputs.
///
/// The inputs are not automatically updated.
/// It can be manually set to simulate inputs, or be automatically updated
/// by another crate (e.g. by the graphics crate).
///
/// # Examples
///
/// See [`Keyboard`], [`Mouse`], [`Fingers`], [`Gamepads`].
#[derive(Default, RootNode, Node, Visit)]
pub struct Inputs {
    /// State of the keyboard.
    pub keyboard: Keyboard,
    /// State of the mouse.
    pub mouse: Mouse,
    /// State of the fingers on touchscreen.
    pub fingers: Fingers,
    /// State of the gamepads.
    pub gamepads: Gamepads,
}
