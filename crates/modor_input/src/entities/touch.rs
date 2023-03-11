use crate::InputState;
use modor_math::Vec2;

/// The state of a finger.
///
/// The entity only exists if the finger is pressed.<br>
/// Once released, the entity remains during one update before being deleted.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn access_touch(fingers: Query<'_, &Finger>) {
///     for finger in fingers.iter() {
///         println!("Position of finger {}: {:?}", finger.id(), finger.position());
///     }
/// }
/// ```
#[derive(Component, NoSystem)]
pub struct Finger {
    id: u64,
    state: InputState,
    position: Vec2,
    delta: Vec2,
}

impl Finger {
    pub(crate) fn new(id: u64) -> Self {
        Self {
            id,
            state: {
                let mut state = InputState::default();
                state.press();
                state
            },
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
        }
    }

    /// Unique identifier of the finger.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// State of the finger.
    pub fn state(&self) -> InputState {
        self.state
    }

    /// Returns the position of the finger in pixels from the top-left corner of the app window.
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the finger position delta in pixels.
    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    pub(crate) fn reset(&mut self) {
        self.state.refresh();
        self.delta = Vec2::ZERO;
    }

    pub(crate) fn update(&mut self, position: Vec2) {
        self.delta.x = position.x - self.position.x;
        self.delta.y = position.y - self.position.y;
        self.position = position;
    }

    pub(crate) fn release(&mut self) {
        self.state.release();
    }
}

/// A touch event.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TouchEvent {
    /// Finger added.
    Started(u64),
    /// Finger removed.
    Ended(u64),
    /// Finger position in pixels from the top-left corner of the app window updated.
    UpdatedPosition(u64, Vec2),
}
