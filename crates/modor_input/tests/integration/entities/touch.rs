use modor::{App, With};
use modor_input::{Finger, InputEventCollector, InputModule, TouchEvent};
use modor_math::Vec2;

#[modor_test(disabled(wasm))]
fn update_state() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(TouchEvent::Started(0, Vec2::ZERO).into());
            c.push(TouchEvent::Started(1, Vec2::ZERO).into());
            c.push(TouchEvent::Started(2, Vec2::ZERO).into());
        })
        .updated()
        .assert_any::<With<Finger>>(3, |e| {
            e.has(|f: &Finger| {
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
        .assert_any::<With<Finger>>(3, |e| {
            e.has(|f: &Finger| {
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
        .assert_any::<With<Finger>>(2, |e| {
            e.has(|f: &Finger| assert_eq!(f.id(), 0))
                .has(|f: &Finger| assert_eq!(f.id(), 1))
        });
}

#[modor_test]
fn update_position() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(TouchEvent::Started(0, Vec2::new(1., 2.)).into());
        })
        .updated()
        .assert::<With<Finger>>(1, |e| {
            e.has(|f: &Finger| {
                assert_eq!(f.id(), 0);
                assert_approx_eq!(f.position(), Vec2::new(1., 2.));
                assert_approx_eq!(f.delta(), Vec2::ZERO);
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
                assert_approx_eq!(f.position(), Vec2::new(2., 5.));
                assert_approx_eq!(f.delta(), Vec2::new(1., 3.));
            })
        })
        .updated()
        .assert::<With<Finger>>(1, |e| {
            e.has(|f: &Finger| {
                assert_eq!(f.id(), 0);
                assert_approx_eq!(f.position(), Vec2::new(2., 5.));
                assert_approx_eq!(f.delta(), Vec2::new(0., 0.));
            })
        });
}
