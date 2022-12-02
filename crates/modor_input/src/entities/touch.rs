use crate::InputState;
use modor::{Built, EntityBuilder};
use modor_math::Vec2;

/// The state of a finger.
///
/// The entity only exists if the finger is pressed.<br>
/// Once released, the entity remains during one update before being deleted.
///
/// # Modor
///
/// - **Type**: entity
/// - **Lifetime**: same as [`InputModule`](crate::InputModule)
///
/// # Examples
///
/// ```rust
/// # use modor::{Single, Query};
/// # use modor_input::Finger;
/// #
/// fn access_touch(fingers: Query<'_, &Finger>) {
///     for finger in fingers.iter() {
///         println!("Position of finger {}: {:?}", finger.id(), finger.position());
///     }
/// }
/// ```
pub struct Finger {
    id: u64,
    state: InputState,
    position: Vec2,
    delta: Vec2,
}

#[entity]
impl Finger {
    /// Unique identifier of the finger.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// State of the finger.
    #[must_use]
    pub fn state(&self) -> InputState {
        self.state
    }

    /// Returns the position of the finger in pixels from the top-left corner of the app window.
    #[must_use]
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Returns the finger position delta in pixels.
    #[must_use]
    pub fn delta(&self) -> Vec2 {
        self.delta
    }

    pub(crate) fn build(id: u64) -> impl Built<Self> {
        EntityBuilder::new(Self {
            id,
            state: {
                let mut state = InputState::default();
                state.press();
                state
            },
            position: Vec2::ZERO,
            delta: Vec2::ZERO,
        })
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
