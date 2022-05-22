use crate::data::{InputDelta, InputState, DEFAULT_INPUT_STATE};
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder};

/// The state of the mouse.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`InputModule`](crate::InputModule)
/// - **Updated during**: [`UpdateInputAction`](crate::UpdateInputAction)
///
/// # Examples
///
/// ```rust
/// # use modor::Single;
/// # use modor_input::{Mouse, MouseButton};
/// #
/// fn access_mouse(mouse: Single<'_, Mouse>) {
///     println!("Position: {:?}", mouse.position());
///     println!("Left button pressed: {:?}", mouse.button(MouseButton::Left).is_pressed());
/// }
/// ```
pub struct Mouse {
    buttons: FxHashMap<MouseButton, InputState>,
    scroll_delta: InputDelta,
    scroll_unit: MouseScrollUnit,
    position: MousePosition,
    delta: InputDelta,
}

#[singleton]
impl Mouse {
    /// Return all pressed buttons.
    pub fn pressed_buttons(&self) -> impl Iterator<Item = MouseButton> + '_ {
        self.buttons
            .iter()
            .filter(|(_, s)| s.is_pressed())
            .map(|(b, _)| *b)
    }

    /// Returns the state of a button.
    pub fn button(&self, button: MouseButton) -> InputState {
        *self.buttons.get(&button).unwrap_or(&DEFAULT_INPUT_STATE)
    }

    /// Returns the scroll delta in pixels.
    ///
    /// The scroll delta can be retrieved in two units: pixels and lines.<br>
    /// In case the delta is retrieved in lines, `row_pixels` and `column_pixels` are used to
    /// make the conversion.
    pub fn scroll_delta_in_pixels(&self, row_pixels: f32, column_pixels: f32) -> InputDelta {
        match self.scroll_unit {
            MouseScrollUnit::Pixel => self.scroll_delta,
            MouseScrollUnit::Line => InputDelta::xy(
                self.scroll_delta.x * row_pixels,
                self.scroll_delta.y * column_pixels,
            ),
        }
    }

    /// Returns the scroll delta in lines.
    ///
    /// The scroll delta can be retrieved in two units: pixels and lines.<br>
    /// In case the delta is retrieved in pixels, `row_pixels` and `column_pixels` are used to
    /// make the conversion.
    pub fn scroll_delta_in_lines(&self, row_pixels: f32, column_pixels: f32) -> InputDelta {
        match self.scroll_unit {
            MouseScrollUnit::Pixel => InputDelta::xy(
                self.scroll_delta.x / row_pixels,
                self.scroll_delta.y / column_pixels,
            ),
            MouseScrollUnit::Line => self.scroll_delta,
        }
    }

    /// Returns the position of the mouse.
    pub fn position(&self) -> MousePosition {
        self.position
    }

    /// Returns the mouse delta.
    ///
    /// The delta does not take into account a possible acceleration created by the system,
    /// in contrary to [`Mouse::position`](crate::Mouse::position).
    pub fn delta(&self) -> InputDelta {
        self.delta
    }

    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            buttons: FxHashMap::default(),
            scroll_delta: InputDelta::default(),
            scroll_unit: MouseScrollUnit::Pixel,
            position: MousePosition::default(),
            delta: InputDelta::default(),
        })
    }

    pub(crate) fn reset(&mut self) {
        for button in self.buttons.values_mut() {
            button.refresh();
        }
        self.scroll_delta = InputDelta::default();
        self.delta = InputDelta::default();
    }

    pub(crate) fn apply_event(&mut self, event: MouseEvent) {
        match event {
            MouseEvent::PressedButton(button) => self.buttons.entry(button).or_default().press(),
            MouseEvent::ReleasedButton(button) => self.buttons.entry(button).or_default().release(),
            MouseEvent::Scroll(delta, unit) => {
                self.scroll_delta += delta;
                self.scroll_unit = unit;
            }
            MouseEvent::UpdatedPosition(position) => self.position = position,
            MouseEvent::Moved(delta) => self.delta += delta,
        }
    }
}

/// A mouse event.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Debug, Clone)]
pub enum MouseEvent {
    /// Button of the mouse pressed.
    PressedButton(MouseButton),
    /// Button of the mouse released.
    ReleasedButton(MouseButton),
    /// Scroll of the mouse detected.
    Scroll(InputDelta, MouseScrollUnit),
    /// Mouse position updated.
    UpdatedPosition(MousePosition),
    /// Mouse moved.
    Moved(InputDelta),
}

/// The position of the mouse in pixels from the top-left corner of the app window.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Default, Clone, Copy, Debug)]
pub struct MousePosition {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
}

impl MousePosition {
    /// Creates a new mouse position.
    pub fn xy(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A mouse button.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle/wheel button.
    Middle,
    /// One of the additional buttons of the mouse.
    Other(u16),
}

/// The unit of the provided mouse scroll delta.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseScrollUnit {
    /// Scroll delta in pixels.
    Pixel,
    /// Scroll delta in lines (rows and columns).
    Line,
}
