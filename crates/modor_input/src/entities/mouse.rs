use crate::data::InputState;
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder};
use modor_math::Vec2;

/// The state of the mouse.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`InputModule`](crate::InputModule)
///
/// # Examples
///
/// ```rust
/// # use modor::Single;
/// # use modor_input::{Mouse, MouseButton};
/// #
/// fn access_mouse(mouse: Single<'_, Mouse>) {
///     println!("Position: {:?}", mouse.position());
///     println!("Left button pressed: {:?}", mouse.button(MouseButton::Left).is_pressed);
/// }
/// ```
pub struct Mouse {
    buttons: FxHashMap<MouseButton, InputState>,
    scroll_delta: Vec2,
    scroll_unit: MouseScrollUnit,
    position: Vec2,
    delta: Vec2,
}

#[singleton]
impl Mouse {
    /// Return all pressed buttons.
    pub fn pressed_buttons(&self) -> impl Iterator<Item = MouseButton> + '_ {
        self.buttons
            .iter()
            .filter(|(_, s)| s.is_pressed)
            .map(|(b, _)| *b)
    }

    /// Returns the state of a button.
    #[must_use]
    pub fn button(&self, button: MouseButton) -> InputState {
        self.buttons.get(&button).copied().unwrap_or_default()
    }

    /// Returns the scroll delta in pixels.
    ///
    /// The scroll delta can be retrieved in two units: pixels and lines.<br>
    /// In case the delta is retrieved in lines, `row_pixels` and `column_pixels` are used to
    /// make the conversion.
    #[must_use]
    pub fn scroll_delta_in_pixels(&self, row_pixels: f32, column_pixels: f32) -> Vec2 {
        match self.scroll_unit {
            MouseScrollUnit::Pixel => self.scroll_delta,
            MouseScrollUnit::Line => Vec2::new(
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
    #[must_use]
    pub fn scroll_delta_in_lines(&self, row_pixels: f32, column_pixels: f32) -> Vec2 {
        match self.scroll_unit {
            MouseScrollUnit::Pixel => Vec2::new(
                self.scroll_delta.x / row_pixels,
                self.scroll_delta.y / column_pixels,
            ),
            MouseScrollUnit::Line => self.scroll_delta,
        }
    }

    /// Returns the position of the mouse in pixels from the top-left corner of the app window.
    #[must_use]
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the mouse delta in pixels.
    ///
    /// The delta does not take into account a possible acceleration created by the system,
    /// in contrary to [`Mouse::position()`](crate::Mouse::position).
    #[must_use]
    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            buttons: FxHashMap::default(),
            scroll_delta: Vec2::ZERO,
            scroll_unit: MouseScrollUnit::Pixel,
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
        })
    }

    pub(crate) fn reset(&mut self) {
        for button in self.buttons.values_mut() {
            button.refresh();
        }
        self.scroll_delta = Vec2::ZERO;
        self.delta = Vec2::ZERO;
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
#[non_exhaustive]
pub enum MouseEvent {
    /// Button of the mouse pressed.
    PressedButton(MouseButton),
    /// Button of the mouse released.
    ReleasedButton(MouseButton),
    /// Scroll of the mouse detected.
    Scroll(Vec2, MouseScrollUnit),
    /// Mouse position in pixels from the top-left corner of the app window updated.
    UpdatedPosition(Vec2),
    /// Mouse moved.
    ///
    /// The mouse delta is in pixels.
    Moved(Vec2),
}

/// A mouse button.
///
/// # Examples
///
/// See [`Mouse`](crate::Mouse).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
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
