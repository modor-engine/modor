use crate::inputs::mappings;
use gilrs::{Axis, Event, EventType, Gilrs};
use log::error;
use modor::App;
use modor_input::{Gamepad, GamepadStick, Inputs};

// coverage: off (inputs cannot be tested)

pub(crate) struct Gamepads {
    gilrs: Option<Gilrs>,
}

impl Gamepads {
    pub(crate) fn new(app: &mut App) -> Self {
        let gilrs = Gilrs::new()
            .map_err(|e| error!("cannot load gamepads: {}", e))
            .ok();
        for gamepad_id in Self::plugged_gamepads_ids(&gilrs) {
            let gamepads = &mut app.get_mut::<Inputs>().gamepads;
            gamepads[gamepad_id].is_connected = true;
        }
        Self { gilrs }
    }

    pub(crate) fn treat_events(&mut self, app: &mut App) {
        let gamepads = &mut app.get_mut::<Inputs>().gamepads;
        while let Some(event) = self.gilrs.as_mut().and_then(Gilrs::next_event) {
            let Event { id, event, .. } = event;
            let id = <_ as Into<usize>>::into(id) as u64;
            Self::apply_event(&mut gamepads[id], event);
        }
        gamepads.sync_d_pad();
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
