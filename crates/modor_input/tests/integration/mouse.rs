use modor_input::{Mouse, MouseButton, MouseScrollDelta};
use modor_math::Vec2;

#[modor_test]
fn create_default() {
    let mouse = Mouse::default();
    assert_eq!(mouse.pressed_iter().count(), 0);
    assert!(!mouse[MouseButton::Left].is_pressed());
    assert!(!mouse[MouseButton::Left].is_just_pressed());
    assert!(!mouse[MouseButton::Left].is_just_released());
}

#[modor_test]
fn press_button() {
    let mut mouse = Mouse::default();
    mouse[MouseButton::Left].press();
    let pressed_buttons: Vec<_> = mouse.pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![MouseButton::Left]);
    assert!(mouse[MouseButton::Left].is_pressed());
    assert!(mouse[MouseButton::Left].is_just_pressed());
    assert!(!mouse[MouseButton::Left].is_just_released());
}

#[modor_test]
fn refresh_after_button_pressed() {
    let mut mouse = Mouse::default();
    mouse[MouseButton::Left].press();
    mouse.refresh();
    let pressed_buttons: Vec<_> = mouse.pressed_iter().collect();
    assert_eq!(pressed_buttons, vec![MouseButton::Left]);
    assert!(mouse[MouseButton::Left].is_pressed());
    assert!(!mouse[MouseButton::Left].is_just_pressed());
    assert!(!mouse[MouseButton::Left].is_just_released());
}

#[modor_test]
fn release_button() {
    let mut mouse = Mouse::default();
    mouse[MouseButton::Left].press();
    mouse.refresh();
    mouse[MouseButton::Left].release();
    assert_eq!(mouse.pressed_iter().count(), 0);
    assert!(!mouse[MouseButton::Left].is_pressed());
    assert!(!mouse[MouseButton::Left].is_just_pressed());
    assert!(mouse[MouseButton::Left].is_just_released());
}

#[modor_test]
fn refresh_after_button_released() {
    let mut mouse = Mouse::default();
    mouse[MouseButton::Left].press();
    mouse.refresh();
    mouse[MouseButton::Left].release();
    mouse.refresh();
    assert_eq!(mouse.pressed_iter().count(), 0);
    assert!(!mouse[MouseButton::Left].is_pressed());
    assert!(!mouse[MouseButton::Left].is_just_pressed());
    assert!(!mouse[MouseButton::Left].is_just_released());
}

#[modor_test]
fn refresh_after_mouse_moved() {
    let mut mouse = Mouse::default();
    mouse.delta = Vec2::new(1., 2.);
    mouse.refresh();
    assert_approx_eq!(mouse.delta, Vec2::ZERO);
}

#[modor_test]
fn refresh_after_scroll() {
    let mut mouse = Mouse::default();
    mouse.scroll_delta = MouseScrollDelta::Pixels(Vec2::new(1., 2.));
    mouse.refresh();
    assert_eq!(mouse.scroll_delta, MouseScrollDelta::default());
}

#[modor_test]
fn retrieve_scroll_delta_when_in_pixels() {
    let mut mouse = Mouse::default();
    mouse.scroll_delta = MouseScrollDelta::Pixels(Vec2::new(20., 50.));
    assert_approx_eq!(mouse.scroll_delta.as_pixels(5., 25.), Vec2::new(20., 50.));
    assert_approx_eq!(mouse.scroll_delta.as_lines(5., 25.), Vec2::new(4., 2.));
}

#[modor_test]
fn retrieve_scroll_delta_when_in_lines() {
    let mut mouse = Mouse::default();
    mouse.scroll_delta = MouseScrollDelta::Lines(Vec2::new(4., 2.));
    assert_approx_eq!(mouse.scroll_delta.as_pixels(5., 25.), Vec2::new(20., 50.));
    assert_approx_eq!(mouse.scroll_delta.as_lines(5., 25.), Vec2::new(4., 2.));
}

#[modor_test]
fn add_scroll_deltas() {
    let mut delta = MouseScrollDelta::Lines(Vec2::new(1., 2.));
    delta += MouseScrollDelta::Lines(Vec2::new(3., 5.));
    assert_approx_eq!(delta.as_lines(0., 0.), Vec2::new(4., 7.));
    let mut delta = MouseScrollDelta::Pixels(Vec2::new(1., 2.));
    delta += MouseScrollDelta::Pixels(Vec2::new(3., 5.));
    assert_approx_eq!(delta.as_pixels(0., 0.), Vec2::new(4., 7.));
    let mut delta = MouseScrollDelta::Pixels(Vec2::new(1., 2.));
    delta += MouseScrollDelta::Lines(Vec2::new(3., 5.));
    assert_approx_eq!(delta.as_lines(0., 0.), Vec2::new(3., 5.));
    let mut delta = MouseScrollDelta::Lines(Vec2::new(1., 2.));
    delta += MouseScrollDelta::Pixels(Vec2::new(3., 5.));
    assert_approx_eq!(delta.as_pixels(0., 0.), Vec2::new(3., 5.));
}
