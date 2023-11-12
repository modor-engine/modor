use crate::input::mappings;
use modor_input::{Fingers, Keyboard, Mouse, MouseScrollDelta};
use modor_math::Vec2;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyEvent, MouseButton, Touch};
use winit::keyboard::PhysicalKey;

#[allow(clippy::cast_possible_truncation)]
pub(crate) fn update_mouse_motion(mouse: &mut Mouse, winit_delta: (f64, f64)) {
    mouse.delta += Vec2::new(winit_delta.0 as f32, -winit_delta.1 as f32);
}

pub(crate) fn update_mouse_button(mouse: &mut Mouse, button: MouseButton, state: ElementState) {
    let button = mappings::to_mouse_button(button);
    match state {
        ElementState::Pressed => mouse[button].press(),
        ElementState::Released => mouse[button].release(),
    }
}

pub(crate) fn update_mouse_wheel(mouse: &mut Mouse, delta: winit::event::MouseScrollDelta) {
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

pub(crate) fn update_mouse_position(mouse: &mut Mouse, position: PhysicalPosition<f64>) {
    mouse.position = winit_pos_to_vec2(position);
}

pub(crate) fn update_keyboard_key(keyboard: &mut Keyboard, event: &KeyEvent) {
    if let PhysicalKey::Code(code) = event.physical_key {
        if let Some(key) = mappings::to_keyboard_key(code) {
            match event.state {
                ElementState::Pressed => keyboard[key].press(),
                ElementState::Released => keyboard[key].release(),
            }
        }
    }
}

pub(crate) fn update_entered_text(keyboard: &mut Keyboard, event: &KeyEvent) {
    if let Some(text) = &event.text {
        keyboard.text += text;
    }
}

pub(crate) fn press_finger(fingers: &mut Fingers, touch: Touch) {
    let finger = &mut fingers[touch.id];
    finger.position = winit_pos_to_vec2(touch.location);
    finger.state.press();
}

pub(crate) fn move_finger(fingers: &mut Fingers, touch: Touch) {
    let finger = &mut fingers[touch.id];
    let position = winit_pos_to_vec2(touch.location);
    finger.delta = position - finger.position;
    finger.position = position;
}

pub(crate) fn release_finger(fingers: &mut Fingers, touch: Touch) {
    let finger = &mut fingers[touch.id];
    finger.state.release();
}

#[allow(clippy::cast_possible_truncation)]
fn winit_pos_to_vec2(position: PhysicalPosition<f64>) -> Vec2 {
    Vec2::new(position.x as f32, position.y as f32)
}
