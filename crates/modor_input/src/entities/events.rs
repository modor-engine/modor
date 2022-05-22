use crate::{Keyboard, KeyboardEvent, Mouse, MouseEvent};
use modor::{Built, EntityBuilder, SingleMut};

/// The input event collector.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`InputModule`](crate::InputModule)
///
/// # Examples
///
/// ```rust
/// # use modor::SingleMut;
/// # use modor_input::{
/// #    InputDelta, InputEvent, InputEventCollector, Key, KeyboardEvent, MouseEvent,
/// #    MouseScrollUnit
/// # };
/// #
/// fn push_events(mut collector: SingleMut<'_, InputEventCollector>) {
///     let scroll_delta = InputDelta::xy(0., 0.5);
///     collector.push(InputEvent::Mouse(MouseEvent::Scroll(scroll_delta, MouseScrollUnit::Line)));
///     collector.push(InputEvent::Keyboard(KeyboardEvent::ReleasedKey(Key::Left)))
/// }
/// ```
pub struct InputEventCollector {
    events: Vec<InputEvent>,
}

#[singleton]
impl InputEventCollector {
    /// Pushes an event.
    pub fn push(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { events: vec![] })
    }

    #[run_as(UpdateInputAction)]
    fn apply(&mut self, mut mouse: SingleMut<'_, Mouse>, mut keyboard: SingleMut<'_, Keyboard>) {
        mouse.reset();
        keyboard.reset();
        for event in self.events.drain(..) {
            match event {
                InputEvent::Mouse(event) => mouse.apply_event(event),
                InputEvent::Keyboard(event) => keyboard.apply_event(event),
            }
        }
    }
}

/// An action done when the input module has treated all events.
#[action]
pub struct UpdateInputAction;

/// An input event.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse event.
    Mouse(MouseEvent),
    /// Keyboard event.
    Keyboard(KeyboardEvent),
}
