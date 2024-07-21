use crate::{Camera2DGlob, Window};
use modor::{App, Builder, GlobRef, Node, RootNodeHandle, Visit};
use modor_input::modor_math::Vec2;
use modor_input::{Finger, InputState, Inputs, MouseButton};

/// A utility type for retrieving cursor properties.
///
/// The cursor corresponds to either the [`Mouse`](modor_input::Mouse) or the first pressed
/// [`Finger`].
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// #[derive(Visit)]
/// struct Cursor {
///     tracker: CursorTracker,
/// }
///
/// impl Node for Cursor {
///     fn on_enter(&mut self, app: &mut App) {
///         println!("Cursor position: {:?}", self.tracker.position(app));
///         println!("Cursor is pressed: {}", self.tracker.state(app).is_pressed());
///     }
/// }
/// ```
#[derive(Visit, Builder)]
pub struct CursorTracker {
    /// The camera used to calculate cursor position.
    ///
    /// Default is the default camera of the [`Window`].
    #[builder(form(value))]
    pub camera: GlobRef<Camera2DGlob>,
    window: RootNodeHandle<Window>,
    inputs: RootNodeHandle<Inputs>,
    is_touch: bool,
    last_finger_id: Option<u64>,
}

impl Node for CursorTracker {
    fn on_enter(&mut self, app: &mut App) {
        let inputs = self.inputs.get(app);
        if let Some((finger_id, _)) = inputs.fingers.pressed_iter().next() {
            self.is_touch = true;
            self.last_finger_id = Some(finger_id);
        } else if inputs.mouse.delta != Vec2::ZERO {
            self.is_touch = false;
        }
    }
}

impl CursorTracker {
    /// Creates a new cursor.
    pub fn new(app: &mut App) -> Self {
        Self {
            camera: app.get_mut::<Window>().camera.glob().to_ref(),
            window: app.handle(),
            inputs: app.handle(),
            is_touch: false,
            last_finger_id: None,
        }
    }

    /// Returns the position of the cursor.
    pub fn position(&self, app: &App) -> Vec2 {
        let window = self.window.get(app);
        let inputs = self.inputs.get(app);
        let window_position = self
            .finger(inputs)
            .map_or(inputs.mouse.position, |finger| finger.position);
        self.camera
            .get(app)
            .world_position(window.size(), window_position)
    }

    /// Returns the state of the cursor.
    ///
    /// For the mouse, [`MouseButton::Left`] state is taken.
    pub fn state(&self, app: &App) -> InputState {
        let inputs = self.inputs.get(app);
        self.finger(inputs)
            .map_or_else(|| inputs.mouse[MouseButton::Left], |finger| finger.state)
    }

    fn finger<'a>(&self, inputs: &'a Inputs) -> Option<&'a Finger> {
        if self.is_touch {
            if let Some((_, finger)) = inputs.fingers.pressed_iter().next() {
                Some(finger)
            } else {
                let finger_id = self
                    .last_finger_id
                    .expect("internal error: no previous finger found");
                Some(&inputs.fingers[finger_id])
            }
        } else {
            None
        }
    }
}
