use modor::testing::TestApp;
use modor::App;
use modor_input::{InputEvent, InputEventCollector, InputModule, Key, Keyboard, KeyboardEvent};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_and_use_keyboard() {
    let mut app: TestApp = App::new().with_entity(InputModule::build()).into();
    app.run_for_singleton(|c: &mut InputEventCollector| {
        c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(Key::A)));
        c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(Key::B)));
        c.push(InputEvent::Keyboard(KeyboardEvent::PressedKey(Key::C)));
        c.push(InputEvent::Keyboard(KeyboardEvent::ReleasedKey(Key::C)));
        c.push(InputEvent::Keyboard(KeyboardEvent::EnteredText(
            "abc".into(),
        )));
        c.push(InputEvent::Keyboard(KeyboardEvent::EnteredText(
            "def".into(),
        )));
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
        assert_eq!(k.text(), "abcdef");
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
        assert_eq!(k.text(), "");
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
