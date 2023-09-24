use modor_input::{Key, Keyboard};
use modor_math::Vec2;

#[modor_test]
fn create_default() {
    let keyboard = Keyboard::default();
    assert_eq!(keyboard.pressed_iter().count(), 0);
    assert!(!keyboard[Key::Return].is_pressed());
    assert!(!keyboard[Key::Return].is_just_pressed());
    assert!(!keyboard[Key::Return].is_just_released());
}

#[modor_test]
fn press_key() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Return].press();
    let pressed_buttons: Vec<_> = keyboard.pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![Key::Return]);
    assert!(keyboard[Key::Return].is_pressed());
    assert!(keyboard[Key::Return].is_just_pressed());
    assert!(!keyboard[Key::Return].is_just_released());
}

#[modor_test]
fn refresh_after_key_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Return].press();
    keyboard.refresh();
    let pressed_buttons: Vec<_> = keyboard.pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![Key::Return]);
    assert!(keyboard[Key::Return].is_pressed());
    assert!(!keyboard[Key::Return].is_just_pressed());
    assert!(!keyboard[Key::Return].is_just_released());
}

#[modor_test]
fn release_key() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Return].press();
    keyboard.refresh();
    keyboard[Key::Return].release();
    assert_eq!(keyboard.pressed_iter().count(), 0);
    assert!(!keyboard[Key::Return].is_pressed());
    assert!(!keyboard[Key::Return].is_just_pressed());
    assert!(keyboard[Key::Return].is_just_released());
}

#[modor_test]
fn refresh_after_key_released() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Return].press();
    keyboard.refresh();
    keyboard[Key::Return].release();
    keyboard.refresh();
    assert_eq!(keyboard.pressed_iter().count(), 0);
    assert!(!keyboard[Key::Return].is_pressed());
    assert!(!keyboard[Key::Return].is_just_pressed());
    assert!(!keyboard[Key::Return].is_just_released());
}

#[modor_test]
fn refresh_after_text_entered() {
    let mut keyboard = Keyboard::default();
    keyboard.text = "entered text".into();
    keyboard.refresh();
    assert_eq!(keyboard.text, "");
}

#[modor_test]
fn retrieve_direction_when_not_pressed() {
    let keyboard = Keyboard::default();
    let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
    assert_approx_eq!(direction, Vec2::ZERO);
}

#[modor_test]
fn retrieve_direction_when_up_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Up].press();
    let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
    assert_approx_eq!(direction, Vec2::Y);
}

#[modor_test]
fn retrieve_direction_when_down_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Down].press();
    let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
    assert_approx_eq!(direction, -Vec2::Y);
}

#[modor_test]
fn retrieve_direction_when_left_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Left].press();
    let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
    assert_approx_eq!(direction, -Vec2::X);
}

#[modor_test]
fn retrieve_direction_when_right_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Right].press();
    let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
    assert_approx_eq!(direction, Vec2::X);
}

#[modor_test]
fn retrieve_direction_when_multiple_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Right].press();
    keyboard[Key::Down].press();
    let direction = keyboard.direction(Key::Left, Key::Right, Key::Up, Key::Down);
    assert_approx_eq!(direction, Vec2::new(1., -1.).with_magnitude(1.).unwrap());
}

#[modor_test]
fn retrieve_axis_when_not_pressed() {
    let keyboard = Keyboard::default();
    let axis = keyboard.axis(Key::Left, Key::Right);
    assert_approx_eq!(axis, 0.);
}

#[modor_test]
fn retrieve_axis_when_left_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Left].press();
    let axis = keyboard.axis(Key::Left, Key::Right);
    assert_approx_eq!(axis, -1.);
}

#[modor_test]
fn retrieve_axis_when_right_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Right].press();
    let axis = keyboard.axis(Key::Left, Key::Right);
    assert_approx_eq!(axis, 1.);
}

#[modor_test]
fn retrieve_axis_when_both_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Left].press();
    keyboard[Key::Right].press();
    let axis = keyboard.axis(Key::Left, Key::Right);
    assert_approx_eq!(axis, 0.);
}
