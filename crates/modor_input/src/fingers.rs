use crate::InputState;
use fxhash::FxHashMap;
use modor_math::Vec2;
use std::ops::{Index, IndexMut};

/// The state of the fingers on touchscreen.
///
/// # Examples
///
/// State access:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn access_fingers(ctx: &mut Context<'_>) {
///     let fingers = &ctx.get_mut::<Inputs>().fingers;
///     println!("Number of registered fingers: {}", fingers.iter().count());
///     println!("Number of pressed fingers: {}", fingers.pressed_iter().count());
///     println!("Finger 0 pressed: {}", fingers[0].state.is_pressed());
///     for (finger_id, finger) in fingers.iter() {
///         println!("Finger {} pressed: {}", finger_id, finger.state.is_pressed());
///     }
/// }
/// ```
///
/// State update:
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// fn modify_fingers(ctx: &mut Context<'_>) {
///     let fingers = &mut ctx.get_mut::<Inputs>().fingers;
///     fingers.refresh();
///     let finger_id = 0;
///     fingers[finger_id].state.press();
/// }
/// ```
#[derive(Debug, Default)]
pub struct Fingers {
    fingers: FxHashMap<u64, Finger>,
}

impl Fingers {
    /// Refreshes fingers state.
    ///
    /// This should be called just before updating the fingers state.
    pub fn refresh(&mut self) {
        for finger in self.fingers.values_mut() {
            finger.refresh();
        }
    }

    /// Returns an iterator on finger IDs and details.
    pub fn iter(&self) -> impl Iterator<Item = (u64, &Finger)> + '_ {
        self.fingers.iter().map(|(&i, f)| (i, f))
    }

    /// Returns an iterator on pressed finger IDs and details.
    pub fn pressed_iter(&self) -> impl Iterator<Item = (u64, &Finger)> + '_ {
        self.iter().filter(|(_, f)| f.state.is_pressed())
    }
}

impl Index<u64> for Fingers {
    type Output = Finger;

    fn index(&self, index: u64) -> &Self::Output {
        self.fingers.get(&index).unwrap_or(&Finger::DEFAULT)
    }
}

impl IndexMut<u64> for Fingers {
    fn index_mut(&mut self, index: u64) -> &mut Self::Output {
        self.fingers.entry(index).or_default()
    }
}

/// The state of a finger.
///
/// # Examples
///
/// See [`Fingers`].
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Finger {
    /// State of the finger.
    pub state: InputState,
    /// Position of the finger.
    pub position: Vec2,
    /// Delta of the finger.
    pub delta: Vec2,
}

impl Finger {
    const DEFAULT: Self = Self {
        state: InputState::DEFAULT,
        position: Vec2::ZERO,
        delta: Vec2::ZERO,
    };

    fn refresh(&mut self) {
        self.state.refresh();
        self.delta = Vec2::ZERO;
    }
}
