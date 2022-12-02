use crate::entities::gamepads::{Gamepad, GamepadEvent};
use crate::{Finger, InputModule, Keyboard, KeyboardEvent, Mouse, MouseEvent, TouchEvent};
use modor::{Built, Entity, EntityBuilder, Query, Single, SingleMut, World};
use modor_math::Vec2;

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
/// # use modor_math::Vec2;
/// # use modor_input::{
/// #    InputEvent, InputEventCollector, Key, KeyboardEvent, MouseEvent,
/// #    MouseScrollUnit, TouchEvent, GamepadEvent, GamepadAxis
/// # };
/// #
/// fn push_events(mut collector: SingleMut<'_, InputEventCollector>) {
///     collector.push(MouseEvent::Scroll(Vec2::new(0., 0.5), MouseScrollUnit::Line).into());
///     collector.push(KeyboardEvent::ReleasedKey(Key::Left).into());
///     collector.push(TouchEvent::Started(10).into());
///     collector.push(TouchEvent::UpdatedPosition(10, Vec2::new(20., 42.)).into());
///     collector.push(GamepadEvent::Plugged(5).into());
///     collector.push(GamepadEvent::UpdatedAxisValue(5, GamepadAxis::LeftStickX, 0.68).into());
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

    #[run]
    fn apply(
        &mut self,
        module: Single<'_, InputModule>,
        mut mouse: SingleMut<'_, Mouse>,
        mut keyboard: SingleMut<'_, Keyboard>,
        mut fingers: Query<'_, (Entity<'_>, &mut Finger)>,
        mut gamepads: Query<'_, (Entity<'_>, &mut Gamepad)>,
        mut world: World<'_>,
    ) {
        Self::delete_released_fingers(&fingers, &mut world);
        mouse.reset();
        keyboard.reset();
        fingers.iter_mut().for_each(|(_, f)| f.reset());
        gamepads.iter_mut().for_each(|(_, g)| g.reset());
        let module_id = module.entity().id();
        for event in self.events.drain(..) {
            trace!("input event `{event:?}` received");
            match event {
                InputEvent::Mouse(event) => mouse.apply_event(event),
                InputEvent::Keyboard(event) => keyboard.apply_event(event),
                InputEvent::Touch(event) => match event {
                    TouchEvent::Started(id) => Self::create_finger(id, module_id, &mut world),
                    TouchEvent::Ended(id) => Self::release_finger(id, &mut fingers),
                    TouchEvent::UpdatedPosition(id, position) => {
                        Self::update_finger(id, position, &mut fingers);
                    }
                },
                InputEvent::Gamepad(event) => match event {
                    GamepadEvent::Plugged(id) => Self::create_gamepad(id, module_id, &mut world),
                    GamepadEvent::Unplugged(id) => Self::delete_gamepad(id, &gamepads, &mut world),
                    event @ (GamepadEvent::PressedButton(..)
                    | GamepadEvent::ReleasedButton(..)
                    | GamepadEvent::UpdatedButtonValue(..)
                    | GamepadEvent::UpdatedAxisValue(..)) => {
                        Self::apply_gamepad_event(event, &mut gamepads);
                    }
                },
            }
        }
        gamepads.iter_mut().for_each(|(_, g)| g.normalize());
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

    fn update_finger(id: u64, position: Vec2, fingers: &mut Query<'_, (Entity<'_>, &mut Finger)>) {
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
            .filter(|(_, t)| t.state().is_just_released)
            .for_each(|(e, _)| world.delete_entity(e.id()));
    }

    fn create_gamepad(id: u64, module_id: usize, world: &mut World<'_>) {
        world.create_child_entity(module_id, Gamepad::build(id));
    }

    fn delete_gamepad(
        id: u64,
        gamepads: &Query<'_, (Entity<'_>, &mut Gamepad)>,
        world: &mut World<'_>,
    ) {
        gamepads
            .iter()
            .filter(|(_, t)| t.id() == id)
            .for_each(|(e, _)| world.delete_entity(e.id()));
    }

    fn apply_gamepad_event(
        event: GamepadEvent,
        gamepads: &mut Query<'_, (Entity<'_>, &mut Gamepad)>,
    ) {
        let gamepad_id = event.id();
        gamepads
            .iter_mut()
            .filter(|(_, f)| f.id() == gamepad_id)
            .for_each(|(_, f)| f.apply_event(&event));
    }
}

/// An input event.
///
/// # Examples
///
/// See [`InputEventCollector`](crate::InputEventCollector).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum InputEvent {
    /// Mouse event.
    Mouse(MouseEvent),
    /// Keyboard event.
    Keyboard(KeyboardEvent),
    /// Touch event.
    Touch(TouchEvent),
    /// Gamepad event.
    Gamepad(GamepadEvent),
}

impl From<MouseEvent> for InputEvent {
    fn from(event: MouseEvent) -> Self {
        Self::Mouse(event)
    }
}

impl From<KeyboardEvent> for InputEvent {
    fn from(event: KeyboardEvent) -> Self {
        Self::Keyboard(event)
    }
}

impl From<TouchEvent> for InputEvent {
    fn from(event: TouchEvent) -> Self {
        Self::Touch(event)
    }
}

impl From<GamepadEvent> for InputEvent {
    fn from(event: GamepadEvent) -> Self {
        Self::Gamepad(event)
    }
}
