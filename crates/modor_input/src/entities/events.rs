use crate::{
    Finger, InputModule, Keyboard, KeyboardEvent, Mouse, MouseEvent, TouchEvent, WindowPosition,
};
use modor::{Built, Entity, EntityBuilder, Query, Single, SingleMut, World};

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
    fn apply(
        &mut self,
        module: Single<'_, InputModule>,
        mut mouse: SingleMut<'_, Mouse>,
        mut keyboard: SingleMut<'_, Keyboard>,
        mut fingers: Query<'_, (Entity<'_>, &mut Finger)>,
        mut world: World<'_>,
    ) {
        Self::delete_released_fingers(&fingers, &mut world);
        mouse.reset();
        keyboard.reset();
        fingers.iter_mut().for_each(|(_, f)| f.reset());
        let module_id = module.entity().id();
        for event in self.events.drain(..) {
            match event {
                InputEvent::Mouse(event) => mouse.apply_event(event),
                InputEvent::Keyboard(event) => keyboard.apply_event(event),
                InputEvent::Touch(event) => match event {
                    TouchEvent::Start(id) => Self::create_finger(id, module_id, &mut world),
                    TouchEvent::End(id) => Self::release_finger(id, &mut fingers),
                    TouchEvent::UpdatedPosition(id, position) => {
                        Self::update_finger(id, position, &mut fingers);
                    }
                },
            }
        }
    }

    fn create_finger(id: u64, module_id: usize, world: &mut World<'_>) {
        world.create_child_entity(module_id, Finger::build(id));
    }

    fn release_finger(id: u64, fingers: &mut Query<'_, (Entity<'_>, &mut Finger)>) {
        fingers
            .iter_mut()
            .filter(|(_, t)| t.id() == id)
            .for_each(|(_, t)| t.release());
    }

    fn update_finger(
        id: u64,
        position: WindowPosition,
        fingers: &mut Query<'_, (Entity<'_>, &mut Finger)>,
    ) {
        fingers
            .iter_mut()
            .filter(|(_, f)| f.id() == id)
            .for_each(|(_, f)| f.update(position));
    }

    fn delete_released_fingers(
        fingers: &Query<'_, (Entity<'_>, &mut Finger)>,
        world: &mut World<'_>,
    ) {
        fingers
            .iter()
            .filter(|(_, t)| t.state().is_just_released())
            .for_each(|(e, _)| world.delete_entity(e.id()));
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
    /// Touch event.
    Touch(TouchEvent),
}
