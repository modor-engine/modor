use crate::data::InputDelta;
use crate::{InputState, WindowPosition};
use modor::{Built, EntityBuilder};

/// The state of a finger.
///
/// The entity only exists if the finger is pressed.<br>
/// Once released, the entity remains during one update before being deleted.
///
/// # Modor
///
/// - **Type**: entity
/// - **Lifetime**: same as [`InputModule`](crate::InputModule)
/// - **Updated during**: [`UpdateInputAction`](crate::UpdateInputAction)
///
/// # Examples
///
/// ```rust
/// # use modor::{Single, Query};
/// # use modor_input::{Finger, MouseButton};
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
    position: WindowPosition,
    delta: InputDelta,
}

#[entity]
impl Finger {
    /// Unique identifier of the finger.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// State of the finger.
    pub fn state(&self) -> InputState {
        self.state
    }

    /// Returns the position of the finger.
    pub fn position(&self) -> WindowPosition {
        self.position
    }

    /// Returns the finger position delta.
    pub fn delta(&self) -> InputDelta {
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
            position: WindowPosition::default(),
            delta: InputDelta::default(),
        })
    }

    pub(crate) fn reset(&mut self) {
        self.state.refresh();
        self.delta = InputDelta::default();
    }

    pub(crate) fn update(&mut self, position: WindowPosition) {
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
pub enum TouchEvent {
    /// Finger added.
    Started(u64),
    /// Finger removed.
    Ended(u64),
    /// Finger position updated.
    UpdatedPosition(u64, WindowPosition),
}
