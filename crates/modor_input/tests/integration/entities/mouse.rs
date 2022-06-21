use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::App;
use modor_input::{
    InputEventCollector, InputModule, Mouse, MouseButton, MouseEvent, MouseScrollUnit,
};
use modor_math::Vec2;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_pressed_buttons() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(MouseEvent::PressedButton(MouseButton::Left).into());
        c.push(MouseEvent::PressedButton(MouseButton::Right).into());
        c.push(MouseEvent::PressedButton(MouseButton::Middle).into());
        c.push(MouseEvent::ReleasedButton(MouseButton::Middle).into());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_eq!(
            m.pressed_buttons().collect::<Vec<_>>(),
            [MouseButton::Left, MouseButton::Right]
        );
        assert!(m.button(MouseButton::Left).is_pressed());
        assert!(m.button(MouseButton::Left).is_just_pressed());
        assert!(!m.button(MouseButton::Left).is_just_released());
        assert!(m.button(MouseButton::Right).is_pressed());
        assert!(m.button(MouseButton::Right).is_just_pressed());
        assert!(!m.button(MouseButton::Right).is_just_released());
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(MouseEvent::ReleasedButton(MouseButton::Right).into());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_eq!(m.pressed_buttons().collect::<Vec<_>>(), [MouseButton::Left]);
        assert!(m.button(MouseButton::Left).is_pressed());
        assert!(!m.button(MouseButton::Left).is_just_pressed());
        assert!(!m.button(MouseButton::Left).is_just_released());
        assert!(!m.button(MouseButton::Right).is_pressed());
        assert!(!m.button(MouseButton::Right).is_just_pressed());
        assert!(m.button(MouseButton::Right).is_just_released());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_eq!(m.pressed_buttons().collect::<Vec<_>>(), [MouseButton::Left]);
        assert!(m.button(MouseButton::Left).is_pressed());
        assert!(!m.button(MouseButton::Left).is_just_pressed());
        assert!(!m.button(MouseButton::Left).is_just_released());
        assert!(!m.button(MouseButton::Right).is_pressed());
        assert!(!m.button(MouseButton::Right).is_just_pressed());
        assert!(!m.button(MouseButton::Right).is_just_released());
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_scroll() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(MouseEvent::Scroll(Vec2::xy(1., 2.), MouseScrollUnit::Line).into());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.scroll_delta_in_lines(2., 3.).x, 1.);
        assert_abs_diff_eq!(m.scroll_delta_in_lines(2., 3.).y, 2.);
        assert_abs_diff_eq!(m.scroll_delta_in_pixels(2., 3.).x, 2.);
        assert_abs_diff_eq!(m.scroll_delta_in_pixels(2., 3.).y, 6.);
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(MouseEvent::Scroll(Vec2::xy(10., 20.), MouseScrollUnit::Pixel).into());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.scroll_delta_in_lines(5., 2.).x, 2.);
        assert_abs_diff_eq!(m.scroll_delta_in_lines(5., 2.).y, 10.);
        assert_abs_diff_eq!(m.scroll_delta_in_pixels(2., 3.).x, 10.);
        assert_abs_diff_eq!(m.scroll_delta_in_pixels(2., 3.).y, 20.);
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.scroll_delta_in_lines(5., 2.).x, 0.);
        assert_abs_diff_eq!(m.scroll_delta_in_lines(5., 2.).y, 0.);
        assert_abs_diff_eq!(m.scroll_delta_in_pixels(2., 3.).x, 0.);
        assert_abs_diff_eq!(m.scroll_delta_in_pixels(2., 3.).y, 0.);
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_position() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(MouseEvent::UpdatedPosition(Vec2::xy(150., 320.)).into());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.position().x, 150.);
        assert_abs_diff_eq!(m.position().y, 320.);
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.position().x, 150.);
        assert_abs_diff_eq!(m.position().y, 320.);
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_delta() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(MouseEvent::Moved(Vec2::xy(18., 15.)).into());
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.delta().x, 18.);
        assert_abs_diff_eq!(m.delta().y, 15.);
    });
    app.update();
    app.assert_singleton::<Mouse>().has(|m: &Mouse| {
        assert_abs_diff_eq!(m.delta().x, 0.);
        assert_abs_diff_eq!(m.delta().y, 0.);
    });
}
