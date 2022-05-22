use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::App;
use modor_input::{InputEvent, InputEventCollector, InputModule, Key, Keyboard, KeyboardEvent};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_pressed_keys() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(Key::A)));
        c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(Key::B)));
        c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(Key::C)));
        c.push(InputEvent::Keyboard(KeyboardEvent::ReleasedKey(Key::C)));
    });
    app.update();
    app.assert_singleton::<Keyboard>().has(|k: &Keyboard| {
        assert_eq!(k.pressed_keys().collect::<Vec<_>>(), [Key::A, Key::B]);
        assert!(k.key(Key::A).is_pressed());
        assert!(k.key(Key::A).is_just_pressed());
        assert!(!k.key(Key::A).is_just_released());
        assert!(k.key(Key::B).is_pressed());
        assert!(k.key(Key::B).is_just_pressed());
        assert!(!k.key(Key::B).is_just_released());
    });
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Keyboard(KeyboardEvent::ReleasedKey(Key::B)));
    });
    app.update();
    app.assert_singleton::<Keyboard>().has(|k: &Keyboard| {
        assert_eq!(k.pressed_keys().collect::<Vec<_>>(), [Key::A]);
        assert!(k.key(Key::A).is_pressed());
        assert!(!k.key(Key::A).is_just_pressed());
        assert!(!k.key(Key::A).is_just_released());
        assert!(!k.key(Key::B).is_pressed());
        assert!(!k.key(Key::B).is_just_pressed());
        assert!(k.key(Key::B).is_just_released());
    });
    app.update();
    app.assert_singleton::<Keyboard>().has(|k: &Keyboard| {
        assert!(k.key(Key::A).is_pressed());
        assert!(!k.key(Key::A).is_just_pressed());
        assert!(!k.key(Key::A).is_just_released());
        assert!(!k.key(Key::B).is_pressed());
        assert!(!k.key(Key::B).is_just_pressed());
        assert!(!k.key(Key::B).is_just_released());
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_text() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Keyboard(KeyboardEvent::EnteredText(
            "abc".into(),
        )));
        c.push(InputEvent::Keyboard(KeyboardEvent::EnteredText(
            "def".into(),
        )));
    });
    app.update();
    app.assert_singleton::<Keyboard>()
        .has(|k: &Keyboard| assert_eq!(k.text(), "abcdef"));
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Keyboard(KeyboardEvent::ReleasedKey(Key::B)));
    });
    app.update();
    app.assert_singleton::<Keyboard>()
        .has(|k: &Keyboard| assert_eq!(k.text(), ""));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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

fn assert_direction(keys: &[Key], direction_x: f32, direction_y: f32) {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        for key in keys {
            c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(*key)));
        }
    });
    app.update();
    app.assert_singleton::<Keyboard>().has(|k: &Keyboard| {
        assert_abs_diff_eq!(
            k.direction(Key::Left, Key::Right, Key::Up, Key::Down).x,
            direction_x
        );
        assert_abs_diff_eq!(
            k.direction(Key::Left, Key::Right, Key::Up, Key::Down).y,
            direction_y
        );
    });
}
