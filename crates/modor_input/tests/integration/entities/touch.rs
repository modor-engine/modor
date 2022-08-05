use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::App;
use modor_input::{Finger, InputEventCollector, InputModule, TouchEvent};
use modor_math::Vec2;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_state() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(TouchEvent::Started(0).into());
        c.push(TouchEvent::Started(1).into());
        c.push(TouchEvent::Started(2).into());
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
        let position = Vec2::new(2., 5.);
        c.push(TouchEvent::UpdatedPosition(1, position).into());
        c.push(TouchEvent::Ended(2).into());
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
        c.push(TouchEvent::Started(0).into());
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert_abs_diff_eq!(f.position(), Vec2::ZERO);
        assert_abs_diff_eq!(f.delta(), Vec2::ZERO);
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        let position = Vec2::new(2., 5.);
        c.push(TouchEvent::UpdatedPosition(0, position).into());
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert_abs_diff_eq!(f.position(), Vec2::new(2., 5.));
        assert_abs_diff_eq!(f.delta(), Vec2::new(2., 5.));
    });
    app.update();
    app.assert_entity(4).has(|f: &Finger| {
        assert_eq!(f.id(), 0);
        assert_abs_diff_eq!(f.position(), Vec2::new(2., 5.));
        assert_abs_diff_eq!(f.delta(), Vec2::new(0., 0.));
    });
}
