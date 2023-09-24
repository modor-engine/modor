use crate::input::mappings;
use crate::runner::app::RunnerApp;
use gilrs::{Axis, Event, EventType, Gilrs};
use modor_input::{Gamepad, GamepadStick};

// coverage: off (gamepads are not easily testable)

pub(crate) struct Gamepads {
    gilrs: Option<Gilrs>,
}

impl Gamepads {
    pub(crate) fn new(app: &mut RunnerApp) -> Self {
        let gilrs = Gilrs::new()
            .map_err(|e| error!("cannot load gamepads: {}", e))
            .ok();
        for gamepad_id in Self::plugged_gamepads_ids(&gilrs) {
            app.update_gamepads(|g| g[gamepad_id].is_connected = true);
        }
        Self { gilrs }
    }

    pub(crate) fn treat_events(&mut self, app: &mut RunnerApp) {
        while let Some(event) = self.gilrs.as_mut().and_then(Gilrs::next_event) {
            let Event { id, event, .. } = event;
            let id = <_ as Into<usize>>::into(id) as u64;
            app.update_gamepads(|g| Self::apply_event(&mut g[id], event));
        }
    }

    fn plugged_gamepads_ids(gilrs: &Option<Gilrs>) -> impl Iterator<Item = u64> + '_ {
        gilrs
            .iter()
            .flat_map(Gilrs::gamepads)
            .map(|(i, _)| <_ as Into<usize>>::into(i) as u64)
    }

    fn apply_event(gamepad: &mut Gamepad, event: EventType) {
        match event {
            EventType::Connected => gamepad.is_connected = true,
            EventType::Disconnected => *gamepad = Gamepad::default(),
            EventType::ButtonPressed(button, _) => {
                if let Some(button) = mappings::to_gamepad_button(button) {
                    gamepad[button].state.press();
                }
            }
            EventType::ButtonReleased(button, _) => {
                if let Some(button) = mappings::to_gamepad_button(button) {
                    gamepad[button].state.release();
                }
            }
            EventType::ButtonChanged(button, value, _) => {
                if let Some(button) = mappings::to_gamepad_button(button) {
                    gamepad[button].value = value;
                }
            }
            EventType::AxisChanged(axis, value, _) => match axis {
                Axis::LeftStickX => gamepad[GamepadStick::LeftStick].x = value,
                Axis::LeftStickY => gamepad[GamepadStick::LeftStick].y = value,
                Axis::RightStickX => gamepad[GamepadStick::RightStick].x = value,
                Axis::RightStickY => gamepad[GamepadStick::RightStick].y = value,
                Axis::DPadX => gamepad[GamepadStick::DPad].x = value,
                Axis::DPadY => gamepad[GamepadStick::DPad].y = value,
                Axis::LeftZ | Axis::RightZ | Axis::Unknown => {}
            },
            EventType::Dropped | EventType::ButtonRepeated(_, _) => {}
        }
    }
}
