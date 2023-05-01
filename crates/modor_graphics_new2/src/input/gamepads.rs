use crate::input::events;
use crate::runner::app::RunnerApp;
use gilrs::{Event, EventType, Gilrs};
use modor_input::{GamepadEvent, InputEvent};

pub(crate) struct Gamepads {
    gilrs: Option<Gilrs>,
}

impl Gamepads {
    pub(crate) fn new(app: &mut RunnerApp) -> Self {
        let gilrs = Gilrs::new()
            .map_err(|e| error!("cannot load gamepads: {}", e))
            .ok();
        for gamepad_id in Self::plugged_gamepads_ids(&gilrs) {
            app.send_event(InputEvent::Gamepad(GamepadEvent::Plugged(gamepad_id)));
        }
        Self { gilrs }
    }

    pub(crate) fn treat_events(&mut self, app: &mut RunnerApp) {
        while let Some(event) = self.gilrs.as_mut().and_then(Gilrs::next_event) {
            let Event { id, event, .. } = event;
            let id = <_ as Into<usize>>::into(id) as u64;
            if let Some(event) = Self::convert_event(event, id) {
                app.send_event(event);
            }
        }
    }

    fn plugged_gamepads_ids(gilrs: &Option<Gilrs>) -> impl Iterator<Item = u64> + '_ {
        gilrs
            .iter()
            .flat_map(Gilrs::gamepads)
            .map(|(i, _)| <_ as Into<usize>>::into(i) as u64)
    }

    fn convert_event(event: EventType, id: u64) -> Option<InputEvent> {
        match event {
            EventType::Connected => Some(InputEvent::Gamepad(GamepadEvent::Plugged(id))),
            EventType::Disconnected => Some(InputEvent::Gamepad(GamepadEvent::Unplugged(id))),
            EventType::ButtonPressed(button, _) => events::pressed_gamepad_button(id, button),
            EventType::ButtonReleased(button, _) => events::released_gamepad_button(id, button),
            EventType::ButtonChanged(button, value, _) => {
                events::changed_gamepad_button(id, button, value)
            }
            EventType::AxisChanged(axis, value, _) => events::changed_gamepad_axis(id, axis, value),
            EventType::Dropped | EventType::ButtonRepeated(_, _) => None,
        }
    }
}
