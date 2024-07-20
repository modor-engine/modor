use modor_input::{GamepadButton, GamepadStick, Gamepads};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;

#[modor::test]
fn create_default() {
    let gamepads = Gamepads::default();
    assert_eq!(gamepads.iter().count(), 0);
    assert_eq!(gamepads[0].pressed_iter().count(), 0);
    assert!(!gamepads[0][GamepadButton::Start].state.is_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_released());
    assert_approx_eq!(gamepads[0][GamepadButton::Start].value, 0.);
    assert_approx_eq!(gamepads[0][GamepadStick::LeftStick], Vec2::ZERO);
}

#[modor::test]
fn press_button() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadButton::Start].state.press();
    gamepads[0][GamepadButton::Start].value = 1.;
    let all_gamepads: Vec<_> = gamepads.iter().map(|(i, _)| i).collect();
    assert_eq!(all_gamepads, vec![0]);
    let pressed_buttons: Vec<_> = gamepads[0].pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![GamepadButton::Start]);
    assert!(gamepads[0][GamepadButton::Start].state.is_pressed());
    assert!(gamepads[0][GamepadButton::Start].state.is_just_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_released());
}

#[modor::test]
fn refresh_after_button_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadButton::Start].state.press();
    gamepads[0][GamepadButton::Start].value = 1.;
    gamepads.refresh();
    let all_gamepads: Vec<_> = gamepads.iter().map(|(i, _)| i).collect();
    assert_eq!(all_gamepads, vec![0]);
    let pressed_buttons: Vec<_> = gamepads[0].pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![GamepadButton::Start]);
    assert!(gamepads[0][GamepadButton::Start].state.is_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_released());
}

#[modor::test]
fn release_button() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadButton::Start].state.press();
    gamepads[0][GamepadButton::Start].value = 1.;
    gamepads.refresh();
    gamepads[0][GamepadButton::Start].state.release();
    gamepads[0][GamepadButton::Start].value = 0.;

    let all_gamepads: Vec<_> = gamepads.iter().map(|(i, _)| i).collect();
    assert_eq!(all_gamepads, vec![0]);
    assert_eq!(gamepads[0].pressed_iter().count(), 0);
    assert!(!gamepads[0][GamepadButton::Start].state.is_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_pressed());
    assert!(gamepads[0][GamepadButton::Start].state.is_just_released());
}

#[modor::test]
fn refresh_after_button_released() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadButton::Start].state.press();
    gamepads[0][GamepadButton::Start].value = 1.;
    gamepads.refresh();
    gamepads[0][GamepadButton::Start].state.release();
    gamepads[0][GamepadButton::Start].value = 0.;
    gamepads.refresh();
    assert_eq!(gamepads.iter().count(), 1);
    assert_eq!(gamepads[0].pressed_iter().count(), 0);
    assert!(!gamepads[0][GamepadButton::Start].state.is_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_pressed());
    assert!(!gamepads[0][GamepadButton::Start].state.is_just_released());
}

#[modor::test]
fn sync_d_pad_when_not_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads.sync_d_pad();
    assert_approx_eq!(gamepads[0][GamepadStick::DPad], Vec2::new(0.5, 0.2));
}

#[modor::test]
fn sync_d_pad_when_previously_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadButton::DPadUp].state.press();
    gamepads.sync_d_pad();
    gamepads[0][GamepadButton::DPadUp].state.release();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads.sync_d_pad();
    assert_approx_eq!(gamepads[0][GamepadStick::DPad], Vec2::ZERO);
}

#[modor::test]
fn sync_d_pad_when_up_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads[0][GamepadButton::DPadUp].state.press();
    gamepads.sync_d_pad();
    assert_approx_eq!(gamepads[0][GamepadStick::DPad], Vec2::Y);
}

#[modor::test]
fn sync_d_pad_when_down_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads[0][GamepadButton::DPadDown].state.press();
    gamepads.sync_d_pad();
    assert_approx_eq!(gamepads[0][GamepadStick::DPad], -Vec2::Y);
}

#[modor::test]
fn sync_d_pad_when_left_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads[0][GamepadButton::DPadLeft].state.press();
    gamepads.sync_d_pad();
    assert_approx_eq!(gamepads[0][GamepadStick::DPad], -Vec2::X);
}

#[modor::test]
fn sync_d_pad_when_right_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads[0][GamepadButton::DPadRight].state.press();
    gamepads.sync_d_pad();
    assert_approx_eq!(gamepads[0][GamepadStick::DPad], Vec2::X);
}

#[modor::test]
fn sync_d_pad_when_multiple_pressed() {
    let mut gamepads = Gamepads::default();
    gamepads[0][GamepadStick::DPad] = Vec2::new(0.5, 0.2);
    gamepads[0][GamepadButton::DPadRight].state.press();
    gamepads[0][GamepadButton::DPadDown].state.press();
    gamepads.sync_d_pad();
    assert_approx_eq!(
        gamepads[0][GamepadStick::DPad],
        Vec2::new(1., -1.).with_magnitude(1.).unwrap()
    );
}
