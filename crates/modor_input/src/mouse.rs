use crate::InputState;
use fxhash::FxHashMap;
use modor_math::Vec2;
use std::ops::{AddAssign, Index, IndexMut};

/// The state of the mouse.
///
/// # Examples
///
/// State access:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn access_mouse(ctx: &mut Context<'_>) {
///     let mouse = &ctx.get_mut::<Inputs>().mouse;
///     println!("Position: {:?}", mouse.position);
///     println!("Position delta: {:?}", mouse.delta);
///     println!("Left button pressed: {}", mouse[MouseButton::Left].is_pressed());
///     println!("Scroll delta: {:?}", mouse.scroll_delta.as_lines(13., 10.));
/// }
/// ```
///
/// State update:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn modify_mouse(ctx: &mut Context<'_>) {
///     let mouse = &mut ctx.get_mut::<Inputs>().mouse;
///     mouse.refresh();
///     mouse[MouseButton::Right].press();
/// }
/// ```
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Mouse {
    /// Position of the mouse in pixels from the top-left corner of the window.
    pub position: Vec2,
    /// Mouse delta in pixels.
    ///
    /// The delta does not take into account a possible acceleration created by the system,
    /// in contrary to [`Mouse::position`](#structfield.position).
    pub delta: Vec2,
    /// Mouse scroll delta.
    pub scroll_delta: MouseScrollDelta,
    buttons: FxHashMap<MouseButton, InputState>,
}

impl Mouse {
    /// Refreshes mouse state.
    ///
    /// This should be called just before updating the mouse state.
    pub fn refresh(&mut self) {
        self.delta = Vec2::ZERO;
        self.scroll_delta = MouseScrollDelta::default();
        for state in self.buttons.values_mut() {
            state.refresh();
        }
    }

    /// Return an iterator on all pressed buttons.
    pub fn pressed_iter(&self) -> impl Iterator<Item = MouseButton> + '_ {
        self.buttons
            .iter()
            .filter(|(_, s)| s.is_pressed())
            .map(|(b, _)| *b)
    }
}

impl Index<MouseButton> for Mouse {
    type Output = InputState;

    fn index(&self, index: MouseButton) -> &Self::Output {
        self.buttons.get(&index).unwrap_or(&InputState::DEFAULT)
    }
}

impl IndexMut<MouseButton> for Mouse {
    fn index_mut(&mut self, index: MouseButton) -> &mut Self::Output {
        self.buttons.entry(index).or_default()
    }
}

/// A mouse button.
///
/// # Examples
///
/// See [`Mouse`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum MouseButton {
    /// Left button.
    Left,
    /// Right button.
    Right,
    /// Middle/wheel button.
    Middle,
    /// Back button.
    ///
    /// Note that this button may not work with all hardware.
    Back,
    /// Forward button.
    ///
    /// Note that this button may not work with all hardware.
    Forward,
    /// One of the additional buttons of the mouse.
    Other(u16),
}

/// The mouse scroll delta.
///
/// # Examples
///
/// See [`Mouse`].
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MouseScrollDelta {
    /// Scroll delta in pixels.
    Pixels(Vec2),
    /// Scroll delta in lines (rows and columns).
    Lines(Vec2),
}

impl AddAssign for MouseScrollDelta {
    fn add_assign(&mut self, rhs: Self) {
        *self = match (*self, rhs) {
            (Self::Lines(delta1), Self::Lines(delta2)) => Self::Lines(delta1 + delta2),
            (Self::Pixels(delta1), Self::Pixels(delta2)) => Self::Pixels(delta1 + delta2),
            (_, delta) => delta,
        };
    }
}

impl Default for MouseScrollDelta {
    fn default() -> Self {
        Self::Pixels(Vec2::ZERO)
    }
}

impl MouseScrollDelta {
    /// Returns the scroll delta in pixels.
    ///
    /// In case the delta is in lines, `row_pixels` and `column_pixels` are used to
    /// make the conversion.
    pub fn as_pixels(self, row_pixels: f32, column_pixels: f32) -> Vec2 {
        match self {
            Self::Pixels(delta) => delta,
            Self::Lines(delta) => delta.with_scale(Vec2::new(row_pixels, column_pixels)),
        }
    }

    /// Returns the scroll delta in lines.
    ///
    /// In case the delta is in pixels, `row_pixels` and `column_pixels` are used to
    /// make the conversion.
    pub fn as_lines(self, row_pixels: f32, column_pixels: f32) -> Vec2 {
        match self {
            Self::Pixels(delta) => delta.with_scale(Vec2::new(1. / row_pixels, 1. / column_pixels)),
            Self::Lines(delta) => delta,
        }
    }
}
