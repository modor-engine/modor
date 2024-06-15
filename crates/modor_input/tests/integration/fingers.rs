use modor_input::Fingers;
use modor_internal::assert_approx_eq;
use modor_math::Vec2;

#[modor::test]
fn create_default() {
    let fingers = Fingers::default();
    assert_eq!(fingers.iter().count(), 0);
    assert_eq!(fingers.pressed_iter().count(), 0);
    assert!(!fingers[0].state.is_pressed());
    assert!(!fingers[0].state.is_just_pressed());
    assert!(!fingers[0].state.is_just_released());
    assert_approx_eq!(fingers[0].position, Vec2::ZERO);
    assert_approx_eq!(fingers[0].delta, Vec2::ZERO);
}

#[modor::test]
fn press_finger() {
    let mut fingers = Fingers::default();
    fingers[0].state.press();
    let all_fingers: Vec<_> = fingers.iter().map(|(i, _)| i).collect();
    assert_eq!(all_fingers, vec![0]);
    let pressed_fingers: Vec<_> = fingers.pressed_iter().map(|(i, _)| i).collect();
    assert_eq!(pressed_fingers, vec![0]);
    assert!(fingers[0].state.is_pressed());
    assert!(fingers[0].state.is_just_pressed());
    assert!(!fingers[0].state.is_just_released());
}

#[modor::test]
fn refresh_after_finger_pressed() {
    let mut fingers = Fingers::default();
    fingers[0].state.press();
    fingers.refresh();
    let all_fingers: Vec<_> = fingers.iter().map(|(i, _)| i).collect();
    assert_eq!(all_fingers, vec![0]);
    let pressed_fingers: Vec<_> = fingers.pressed_iter().map(|(i, _)| i).collect();
    assert_eq!(pressed_fingers, vec![0]);
    assert!(fingers[0].state.is_pressed());
    assert!(!fingers[0].state.is_just_pressed());
    assert!(!fingers[0].state.is_just_released());
}

#[modor::test]
fn release_finger() {
    let mut fingers = Fingers::default();
    fingers[0].state.press();
    fingers.refresh();
    fingers[0].state.release();
    let all_fingers: Vec<_> = fingers.iter().map(|(i, _)| i).collect();
    assert_eq!(all_fingers, vec![0]);
    assert_eq!(fingers.pressed_iter().count(), 0);
    assert!(!fingers[0].state.is_pressed());
    assert!(!fingers[0].state.is_just_pressed());
    assert!(fingers[0].state.is_just_released());
}

#[modor::test]
fn refresh_after_finger_released() {
    let mut fingers = Fingers::default();
    fingers[0].state.press();
    fingers.refresh();
    fingers[0].state.release();
    fingers.refresh();
    let all_fingers: Vec<_> = fingers.iter().map(|(i, _)| i).collect();
    assert_eq!(all_fingers, vec![0]);
    assert_eq!(fingers.pressed_iter().count(), 0);
    assert!(!fingers[0].state.is_pressed());
    assert!(!fingers[0].state.is_just_pressed());
    assert!(!fingers[0].state.is_just_released());
}

#[modor::test]
fn refresh_after_finger_moved() {
    let mut fingers = Fingers::default();
    fingers[0].delta = Vec2::new(1., 2.);
    fingers.refresh();
    assert_approx_eq!(fingers[0].delta, Vec2::ZERO);
}
