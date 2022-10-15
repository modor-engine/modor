use approx::assert_abs_diff_eq;
use modor::{App, With};
use modor_input::{Finger, InputEventCollector, InputModule, TouchEvent};
use modor_math::Vec2;

#[test]
fn update_state() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(TouchEvent::Started(0).into());
            c.push(TouchEvent::Started(1).into());
            c.push(TouchEvent::Started(2).into());
        })
        .updated()
        .assert::<With<Finger>>(3, |e| {
            e.any()
                .has(|f: &Finger| {
                    assert_eq!(f.id(), 0);
                    assert!(f.state().is_pressed);
                    assert!(f.state().is_just_pressed);
                    assert!(!f.state().is_just_released);
                })
                .has(|f: &Finger| {
                    assert_eq!(f.id(), 1);
                    assert!(f.state().is_pressed);
                    assert!(f.state().is_just_pressed);
                    assert!(!f.state().is_just_released);
                })
                .has(|f: &Finger| {
                    assert_eq!(f.id(), 2);
                    assert!(f.state().is_pressed);
                    assert!(f.state().is_just_pressed);
                    assert!(!f.state().is_just_released);
                })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            let position = Vec2::new(2., 5.);
            c.push(TouchEvent::UpdatedPosition(1, position).into());
            c.push(TouchEvent::Ended(2).into());
        })
        .updated()
        .assert::<With<Finger>>(3, |e| {
            e.any()
                .has(|f: &Finger| {
                    assert_eq!(f.id(), 0);
                    assert!(f.state().is_pressed);
                    assert!(!f.state().is_just_pressed);
                    assert!(!f.state().is_just_released);
                })
                .has(|f: &Finger| {
                    assert_eq!(f.id(), 1);
                    assert!(f.state().is_pressed);
                    assert!(!f.state().is_just_pressed);
                    assert!(!f.state().is_just_released);
                })
                .has(|f: &Finger| {
                    assert_eq!(f.id(), 2);
                    assert!(!f.state().is_pressed);
                    assert!(!f.state().is_just_pressed);
                    assert!(f.state().is_just_released);
                })
        })
        .updated()
        .assert::<With<Finger>>(2, |e| {
            e.any()
                .has(|f: &Finger| assert_eq!(f.id(), 0))
                .has(|f: &Finger| assert_eq!(f.id(), 1))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_position() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| c.push(TouchEvent::Started(0).into()))
        .updated()
        .assert::<With<Finger>>(1, |e| {
            e.has(|f: &Finger| {
                assert_eq!(f.id(), 0);
                assert_abs_diff_eq!(f.position(), Vec2::ZERO);
                assert_abs_diff_eq!(f.delta(), Vec2::ZERO);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            let position = Vec2::new(2., 5.);
            c.push(TouchEvent::UpdatedPosition(0, position).into());
        })
        .updated()
        .assert::<With<Finger>>(1, |e| {
            e.has(|f: &Finger| {
                assert_eq!(f.id(), 0);
                assert_abs_diff_eq!(f.position(), Vec2::new(2., 5.));
                assert_abs_diff_eq!(f.delta(), Vec2::new(2., 5.));
            })
        })
        .updated()
        .assert::<With<Finger>>(1, |e| {
            e.has(|f: &Finger| {
                assert_eq!(f.id(), 0);
                assert_abs_diff_eq!(f.position(), Vec2::new(2., 5.));
                assert_abs_diff_eq!(f.delta(), Vec2::new(0., 0.));
            })
        });
}
