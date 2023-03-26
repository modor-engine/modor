use crate::input::mappings;
use modor_input::{
    GamepadEvent, InputEvent, KeyboardEvent, MouseEvent, MouseScrollUnit, TouchEvent,
};
use modor_math::Vec2;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, Touch};

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn mouse_motion(winit_delta: (f64, f64)) -> InputEvent {
    let delta = Vec2::new(winit_delta.0 as f32, -winit_delta.1 as f32);
    InputEvent::Mouse(MouseEvent::Moved(delta))
}

pub(crate) fn mouse_button(button: MouseButton, state: ElementState) -> InputEvent {
    let button = mappings::to_mouse_button(button);
    InputEvent::Mouse(match state {
        ElementState::Pressed => MouseEvent::PressedButton(button),
        ElementState::Released => MouseEvent::ReleasedButton(button),
    })
}

pub(crate) fn mouse_wheel(delta: MouseScrollDelta) -> InputEvent {
    InputEvent::Mouse(match delta {
        MouseScrollDelta::LineDelta(columns, rows) => {
            let delta = Vec2::new(columns, -rows);
            MouseEvent::Scroll(delta, MouseScrollUnit::Line)
        }
        MouseScrollDelta::PixelDelta(delta) => {
            let delta = winit_pos_to_vec2(delta);
            MouseEvent::Scroll(Vec2::new(delta.x, -delta.y), MouseScrollUnit::Pixel)
        }
    })
}

pub(crate) fn mouse_position(position: PhysicalPosition<f64>) -> InputEvent {
    let position = winit_pos_to_vec2(position);
    InputEvent::Mouse(MouseEvent::UpdatedPosition(position))
}

pub(crate) fn keyboard_key(input: KeyboardInput) -> Option<InputEvent> {
    if let Some(code) = input.virtual_keycode {
        let key = mappings::to_keyboard_key(code);
        Some(InputEvent::Keyboard(match input.state {
            ElementState::Pressed => KeyboardEvent::PressedKey(key),
            ElementState::Released => KeyboardEvent::ReleasedKey(key),
        }))
    } else {
        None
    }
}

pub(crate) fn character(character: char) -> InputEvent {
    InputEvent::Keyboard(KeyboardEvent::EnteredText(character.into()))
}

pub(crate) fn started_touch(touch: Touch) -> [InputEvent; 2] {
    let position = winit_pos_to_vec2(touch.location);
    [
        InputEvent::Touch(TouchEvent::Started(touch.id)),
        InputEvent::Touch(TouchEvent::UpdatedPosition(touch.id, position)),
    ]
}

pub(crate) fn moved_touch(touch: Touch) -> InputEvent {
    let position = winit_pos_to_vec2(touch.location);
    InputEvent::Touch(TouchEvent::UpdatedPosition(touch.id, position))
}

pub(crate) fn ended_touch(touch: Touch) -> InputEvent {
    InputEvent::Touch(TouchEvent::Ended(touch.id))
}

pub(crate) fn pressed_gamepad_button(gamepad_id: u64, button: gilrs::Button) -> Option<InputEvent> {
    if let Some(button) = mappings::to_gamepad_button(button) {
        Some(InputEvent::Gamepad(GamepadEvent::PressedButton(
            gamepad_id, button,
        )))
    } else {
        None
    }
}

pub(crate) fn released_gamepad_button(
    gamepad_id: u64,
    button: gilrs::Button,
) -> Option<InputEvent> {
    if let Some(button) = mappings::to_gamepad_button(button) {
        Some(InputEvent::Gamepad(GamepadEvent::ReleasedButton(
            gamepad_id, button,
        )))
    } else {
        None
    }
}

pub(crate) fn changed_gamepad_button(
    gamepad_id: u64,
    button: gilrs::Button,
    value: f32,
) -> Option<InputEvent> {
    if let Some(button) = mappings::to_gamepad_button(button) {
        Some(InputEvent::Gamepad(GamepadEvent::UpdatedButtonValue(
            gamepad_id, button, value,
        )))
    } else {
        None
    }
}

pub(crate) fn changed_gamepad_axis(
    gamepad_id: u64,
    axis: gilrs::Axis,
    value: f32,
) -> Option<InputEvent> {
    if let Some(axis) = mappings::to_gamepad_axis(axis) {
        Some(InputEvent::Gamepad(GamepadEvent::UpdatedAxisValue(
            gamepad_id, axis, value,
        )))
    } else {
        None
    }
}

#[allow(clippy::cast_possible_truncation)]
fn winit_pos_to_vec2(position: PhysicalPosition<f64>) -> Vec2 {
    Vec2::new(position.x as f32, position.y as f32)
}
