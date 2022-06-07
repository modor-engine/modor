use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::App;
use modor_input::{
    Finger, InputEvent, InputEventCollector, InputModule, TouchEvent, WindowPosition,
};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_state() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Touch(TouchEvent::Start(0)));
        c.push(InputEvent::Touch(TouchEvent::Start(1)));
        c.push(InputEvent::Touch(TouchEvent::Start(2)));
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert!(f.state().is_pressed());
        assert!(f.state().is_just_pressed());
        assert!(!f.state().is_just_released());
    });
    app.assert_entity(5).has(|f: &Finger| {
        assert_eq!(f.id(), 1);
        assert!(f.state().is_pressed());
        assert!(f.state().is_just_pressed());
        assert!(!f.state().is_just_released());
    });
    app.assert_entity(6).has(|f: &Finger| {
        assert_eq!(f.id(), 2);
        assert!(f.state().is_pressed());
        assert!(f.state().is_just_pressed());
        assert!(!f.state().is_just_released());
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        let position = WindowPosition::xy(2., 5.);
        c.push(InputEvent::Touch(TouchEvent::UpdatedPosition(1, position)));
        c.push(InputEvent::Touch(TouchEvent::End(2)));
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert!(f.state().is_pressed());
        assert!(!f.state().is_just_pressed());
        assert!(!f.state().is_just_released());
    });
    app.assert_entity(5).has(|f: &Finger| {
        assert_eq!(f.id(), 1);
        assert!(f.state().is_pressed());
        assert!(!f.state().is_just_pressed());
        assert!(!f.state().is_just_released());
    });
    app.assert_entity(6).has(|f: &Finger| {
        assert_eq!(f.id(), 2);
        assert!(!f.state().is_pressed());
        assert!(!f.state().is_just_pressed());
        assert!(f.state().is_just_released());
    });
    app.update();
    app.assert_entity(6).does_not_exist();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_position() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Touch(TouchEvent::Start(0)));
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert_abs_diff_eq!(f.position().x, 0.);
        assert_abs_diff_eq!(f.position().y, 0.);
        assert_abs_diff_eq!(f.delta().x, 0.);
        assert_abs_diff_eq!(f.delta().y, 0.);
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        let position = WindowPosition::xy(2., 5.);
        c.push(InputEvent::Touch(TouchEvent::UpdatedPosition(0, position)));
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert_abs_diff_eq!(f.position().x, 2.);
        assert_abs_diff_eq!(f.position().y, 5.);
        assert_abs_diff_eq!(f.delta().x, 2.);
        assert_abs_diff_eq!(f.delta().y, 5.);
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert_abs_diff_eq!(f.position().x, 2.);
        assert_abs_diff_eq!(f.position().y, 5.);
        assert_abs_diff_eq!(f.delta().x, 0.);
        assert_abs_diff_eq!(f.delta().y, 0.);
    });
}
