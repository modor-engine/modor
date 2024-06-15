use crate::inputs::mappings;
use modor::App;
use modor_input::modor_math::Vec2;
use modor_input::{Inputs, MouseScrollDelta};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, Touch, TouchPhase};
use winit::keyboard::PhysicalKey;

// coverage: off (inputs cannot be tested)

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn update_mouse_motion(app: &mut Option<App>, delta: (f64, f64)) {
    let Some(app) = app.as_mut() else { return };
    let mouse = &mut app.get_mut::<Inputs>().mouse;
    mouse.delta += Vec2::new(delta.0 as f32, -delta.1 as f32);
}

pub(crate) fn update_mouse_button(app: &mut Option<App>, button: MouseButton, state: ElementState) {
    let Some(app) = app.as_mut() else { return };
    let mouse = &mut app.get_mut::<Inputs>().mouse;
    let button = mappings::to_mouse_button(button);
    match state {
        ElementState::Pressed => mouse[button].press(),
        ElementState::Released => mouse[button].release(),
    }
}

pub(crate) fn update_mouse_wheel(app: &mut Option<App>, delta: winit::event::MouseScrollDelta) {
    let Some(app) = app.as_mut() else { return };
    let mouse = &mut app.get_mut::<Inputs>().mouse;
    mouse.scroll_delta += match delta {
        winit::event::MouseScrollDelta::LineDelta(columns, rows) => {
            MouseScrollDelta::Lines(Vec2::new(columns, -rows))
        }
        winit::event::MouseScrollDelta::PixelDelta(delta) => {
            let delta = winit_pos_to_vec2(delta);
            MouseScrollDelta::Pixels(Vec2::new(delta.x, -delta.y))
        }
    };
}

pub(crate) fn update_mouse_position(app: &mut Option<App>, position: PhysicalPosition<f64>) {
    let Some(app) = app.as_mut() else { return };
    let mouse = &mut app.get_mut::<Inputs>().mouse;
    mouse.position = winit_pos_to_vec2(position);
}

pub(crate) fn update_keyboard_key(app: &mut Option<App>, event: KeyEvent) {
    let Some(app) = app.as_mut() else { return };
    let keyboard = &mut app.get_mut::<Inputs>().keyboard;
    if let PhysicalKey::Code(code) = event.physical_key {
        if let Some(key) = mappings::to_keyboard_key(code) {
            match event.state {
                ElementState::Pressed => keyboard[key].press(),
                ElementState::Released => keyboard[key].release(),
            }
        }
    }
    if let Some(text) = &event.text {
        keyboard.text += text;
    }
}

pub(crate) fn update_fingers(app: &mut Option<App>, touch: Touch) {
    let Some(app) = app.as_mut() else { return };
    let fingers = &mut app.get_mut::<Inputs>().fingers;
    let finger = &mut fingers[touch.id];
    match touch.phase {
        TouchPhase::Started => {
            finger.position = winit_pos_to_vec2(touch.location);
            finger.state.press();
        }
        TouchPhase::Moved => {
            let position = winit_pos_to_vec2(touch.location);
            finger.delta = position - finger.position;
            finger.position = position;
        }
        TouchPhase::Ended | TouchPhase::Cancelled => {
            finger.state.release();
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn winit_pos_to_vec2(position: PhysicalPosition<f64>) -> Vec2 {
    Vec2::new(position.x as f32, position.y as f32)
}
