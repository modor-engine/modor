use modor::{App, With};
use modor_input::{InputEventCollector, InputModule, Key, Keyboard, KeyboardEvent};
use modor_math::Vec2;

#[modor_test]
fn update_pressed_keys() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(KeyboardEvent::PressedKey(Key::A).into());
            c.push(KeyboardEvent::PressedKey(Key::B).into());
            c.push(KeyboardEvent::PressedKey(Key::C).into());
            c.push(KeyboardEvent::ReleasedKey(Key::C).into());
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| {
            e.has(|k: &Keyboard| {
                assert_eq!(k.pressed_keys().collect::<Vec<_>>(), [Key::A, Key::B]);
                assert!(k.key(Key::A).is_pressed);
                assert!(k.key(Key::A).is_just_pressed);
                assert!(!k.key(Key::A).is_just_released);
                assert!(k.key(Key::B).is_pressed);
                assert!(k.key(Key::B).is_just_pressed);
                assert!(!k.key(Key::B).is_just_released);
            })
        })
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(KeyboardEvent::ReleasedKey(Key::B).into());
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| {
            e.has(|k: &Keyboard| {
                assert_eq!(k.pressed_keys().collect::<Vec<_>>(), [Key::A]);
                assert!(k.key(Key::A).is_pressed);
                assert!(!k.key(Key::A).is_just_pressed);
                assert!(!k.key(Key::A).is_just_released);
                assert!(!k.key(Key::B).is_pressed);
                assert!(!k.key(Key::B).is_just_pressed);
                assert!(k.key(Key::B).is_just_released);
            })
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| {
            e.has(|k: &Keyboard| {
                assert!(k.key(Key::A).is_pressed);
                assert!(!k.key(Key::A).is_just_pressed);
                assert!(!k.key(Key::A).is_just_released);
                assert!(!k.key(Key::B).is_pressed);
                assert!(!k.key(Key::B).is_just_pressed);
                assert!(!k.key(Key::B).is_just_released);
            })
        });
}

#[modor_test]
fn update_text() {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(KeyboardEvent::EnteredText("abc".into()).into());
            c.push(KeyboardEvent::EnteredText("def".into()).into());
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| e.has(|k: &Keyboard| assert_eq!(k.text(), "abcdef")))
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            c.push(KeyboardEvent::ReleasedKey(Key::B).into());
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| e.has(|k: &Keyboard| assert_eq!(k.text(), "")));
}

#[modor_test]
fn calculate_direction() {
    let diagonal = 1. / 2.0f32.sqrt();
    assert_direction(&[Key::Left], -1., 0.);
    assert_direction(&[Key::Right], 1., 0.);
    assert_direction(&[Key::Up], 0., 1.);
    assert_direction(&[Key::Down], 0., -1.);
    assert_direction(&[Key::Left, Key::Up], -diagonal, diagonal);
    assert_direction(&[Key::Left, Key::Down], -diagonal, -diagonal);
    assert_direction(&[Key::Right, Key::Up], diagonal, diagonal);
    assert_direction(&[Key::Right, Key::Down], diagonal, -diagonal);
    assert_direction(&[Key::Left, Key::Right], 0., 0.);
    assert_direction(&[Key::Up, Key::Down], 0., 0.);
    assert_direction(&[Key::Up, Key::Down, Key::Left], -1., 0.);
    assert_direction(&[Key::Up, Key::Down, Key::Right], 1., 0.);
    assert_direction(&[Key::Left, Key::Right, Key::Up], 0., 1.);
    assert_direction(&[Key::Left, Key::Right, Key::Down], 0., -1.);
    assert_direction(&[Key::Left, Key::Right, Key::Up, Key::Down], 0., 0.);
}

#[modor_test]
fn calculate_axis() {
    assert_axis(&[], 0.);
    assert_axis(&[Key::Left], -1.);
    assert_axis(&[Key::Right], 1.);
    assert_axis(&[Key::Left, Key::Right], 0.);
}

fn assert_direction(keys: &[Key], direction_x: f32, direction_y: f32) {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            for key in keys {
                c.push(KeyboardEvent::PressedKey(*key).into());
            }
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| {
            e.has(|k: &Keyboard| {
                assert_approx_eq!(
                    k.direction(Key::Left, Key::Right, Key::Up, Key::Down),
                    Vec2::new(direction_x, direction_y)
                );
            })
        });
}

fn assert_axis(keys: &[Key], axis: f32) {
    App::new()
        .with_entity(InputModule::build())
        .with_update::<(), _>(|c: &mut InputEventCollector| {
            for key in keys {
                c.push(KeyboardEvent::PressedKey(*key).into());
            }
        })
        .updated()
        .assert::<With<Keyboard>>(1, |e| {
            e.has(|k: &Keyboard| assert_approx_eq!(k.axis(Key::Left, Key::Right), axis))
        });
}
