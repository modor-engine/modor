use modor::{App, With};
use modor_input::{
    InputEventCollector, InputModule, Mouse, MouseButton, MouseEvent, MouseScrollUnit,
};
use modor_math::Vec2;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_pressed_buttons() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(MouseEvent::PressedButton(MouseButton::Left).into());
            c.push(MouseEvent::PressedButton(MouseButton::Right).into());
            c.push(MouseEvent::PressedButton(MouseButton::Middle).into());
            c.push(MouseEvent::ReleasedButton(MouseButton::Middle).into());
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| {
                let pressed_buttons = m.pressed_buttons().collect::<Vec<_>>();
                assert_eq!(pressed_buttons, [MouseButton::Left, MouseButton::Right]);
                assert!(m.button(MouseButton::Left).is_pressed);
                assert!(m.button(MouseButton::Left).is_just_pressed);
                assert!(!m.button(MouseButton::Left).is_just_released);
                assert!(m.button(MouseButton::Right).is_pressed);
                assert!(m.button(MouseButton::Right).is_just_pressed);
                assert!(!m.button(MouseButton::Right).is_just_released);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(MouseEvent::ReleasedButton(MouseButton::Right).into());
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| {
                assert_eq!(m.pressed_buttons().collect::<Vec<_>>(), [MouseButton::Left]);
                assert!(m.button(MouseButton::Left).is_pressed);
                assert!(!m.button(MouseButton::Left).is_just_pressed);
                assert!(!m.button(MouseButton::Left).is_just_released);
                assert!(!m.button(MouseButton::Right).is_pressed);
                assert!(!m.button(MouseButton::Right).is_just_pressed);
                assert!(m.button(MouseButton::Right).is_just_released);
            })
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| {
                assert_eq!(m.pressed_buttons().collect::<Vec<_>>(), [MouseButton::Left]);
                assert!(m.button(MouseButton::Left).is_pressed);
                assert!(!m.button(MouseButton::Left).is_just_pressed);
                assert!(!m.button(MouseButton::Left).is_just_released);
                assert!(!m.button(MouseButton::Right).is_pressed);
                assert!(!m.button(MouseButton::Right).is_just_pressed);
                assert!(!m.button(MouseButton::Right).is_just_released);
            })
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_scroll() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(MouseEvent::Scroll(Vec2::new(1., 2.), MouseScrollUnit::Line).into());
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| {
                assert_approx_eq!(m.scroll_delta_in_lines(2., 3.), Vec2::new(1., 2.));
                assert_approx_eq!(m.scroll_delta_in_pixels(2., 3.), Vec2::new(2., 6.));
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(MouseEvent::Scroll(Vec2::new(10., 20.), MouseScrollUnit::Pixel).into());
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| {
                assert_approx_eq!(m.scroll_delta_in_lines(5., 2.), Vec2::new(2., 10.));
                assert_approx_eq!(m.scroll_delta_in_pixels(2., 3.), Vec2::new(10., 20.));
            })
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| {
                assert_approx_eq!(m.scroll_delta_in_lines(5., 2.), Vec2::ZERO);
                assert_approx_eq!(m.scroll_delta_in_pixels(2., 3.), Vec2::ZERO);
            })
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_position() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(MouseEvent::UpdatedPosition(Vec2::new(150., 320.)).into());
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| assert_approx_eq!(m.position(), Vec2::new(150., 320.)))
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| assert_approx_eq!(m.position(), Vec2::new(150., 320.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_delta() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(MouseEvent::Moved(Vec2::new(18., 15.)).into());
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| assert_approx_eq!(m.delta(), Vec2::new(18., 15.)))
        })
        .updated()
        .assert::<With<Mouse>>(1, |e| {
            e.has(|m: &Mouse| assert_approx_eq!(m.delta(), Vec2::ZERO))
        });
}
