use modor_input::{Key, Keyboard};
use modor_math::Vec2;

#[modor_test]
fn create_default() {
    let keyboard = Keyboard::default();
    assert_eq!(keyboard.pressed_iter().count(), 0);
    assert!(!keyboard[Key::Enter].is_pressed());
    assert!(!keyboard[Key::Enter].is_just_pressed());
    assert!(!keyboard[Key::Enter].is_just_released());
}

#[modor_test]
fn press_key() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Enter].press();
    let pressed_buttons: Vec<_> = keyboard.pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![Key::Enter]);
    assert!(keyboard[Key::Enter].is_pressed());
    assert!(keyboard[Key::Enter].is_just_pressed());
    assert!(!keyboard[Key::Enter].is_just_released());
}

#[modor_test]
fn refresh_after_key_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Enter].press();
    keyboard.refresh();
    let pressed_buttons: Vec<_> = keyboard.pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![Key::Enter]);
    assert!(keyboard[Key::Enter].is_pressed());
    assert!(!keyboard[Key::Enter].is_just_pressed());
    assert!(!keyboard[Key::Enter].is_just_released());
}

#[modor_test]
fn release_key() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Enter].press();
    keyboard.refresh();
    keyboard[Key::Enter].release();
    assert_eq!(keyboard.pressed_iter().count(), 0);
    assert!(!keyboard[Key::Enter].is_pressed());
    assert!(!keyboard[Key::Enter].is_just_pressed());
    assert!(keyboard[Key::Enter].is_just_released());
}

#[modor_test]
fn refresh_after_key_released() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::Enter].press();
    keyboard.refresh();
    keyboard[Key::Enter].release();
    keyboard.refresh();
    assert_eq!(keyboard.pressed_iter().count(), 0);
    assert!(!keyboard[Key::Enter].is_pressed());
    assert!(!keyboard[Key::Enter].is_just_pressed());
    assert!(!keyboard[Key::Enter].is_just_released());
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
    let direction = keyboard.direction(
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    );
    assert_approx_eq!(direction, Vec2::ZERO);
}

#[modor_test]
fn retrieve_direction_when_up_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowUp].press();
    let direction = keyboard.direction(
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    );
    assert_approx_eq!(direction, Vec2::Y);
}

#[modor_test]
fn retrieve_direction_when_down_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowDown].press();
    let direction = keyboard.direction(
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    );
    assert_approx_eq!(direction, -Vec2::Y);
}

#[modor_test]
fn retrieve_direction_when_left_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowLeft].press();
    let direction = keyboard.direction(
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    );
    assert_approx_eq!(direction, -Vec2::X);
}

#[modor_test]
fn retrieve_direction_when_right_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowRight].press();
    let direction = keyboard.direction(
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    );
    assert_approx_eq!(direction, Vec2::X);
}

#[modor_test]
fn retrieve_direction_when_multiple_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowRight].press();
    keyboard[Key::ArrowDown].press();
    let direction = keyboard.direction(
        Key::ArrowLeft,
        Key::ArrowRight,
        Key::ArrowUp,
        Key::ArrowDown,
    );
    assert_approx_eq!(direction, Vec2::new(1., -1.).with_magnitude(1.).unwrap());
}

#[modor_test]
fn retrieve_axis_when_not_pressed() {
    let keyboard = Keyboard::default();
    let axis = keyboard.axis(Key::ArrowLeft, Key::ArrowRight);
    assert_approx_eq!(axis, 0.);
}

#[modor_test]
fn retrieve_axis_when_left_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowLeft].press();
    let axis = keyboard.axis(Key::ArrowLeft, Key::ArrowRight);
    assert_approx_eq!(axis, -1.);
}

#[modor_test]
fn retrieve_axis_when_right_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowRight].press();
    let axis = keyboard.axis(Key::ArrowLeft, Key::ArrowRight);
    assert_approx_eq!(axis, 1.);
}

#[modor_test]
fn retrieve_axis_when_both_pressed() {
    let mut keyboard = Keyboard::default();
    keyboard[Key::ArrowLeft].press();
    keyboard[Key::ArrowRight].press();
    let axis = keyboard.axis(Key::ArrowLeft, Key::ArrowRight);
    assert_approx_eq!(axis, 0.);
}
