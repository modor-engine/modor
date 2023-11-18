use modor::{App, With};
use modor_input::VirtualKeyboard;

#[modor_test]
fn create_default() {
    App::new()
        .with_entity(modor_input::module())
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(!k.is_open())))
        .updated()
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(!k.is_open())));
}

#[modor_test]
fn open() {
    App::new()
        .with_entity(modor_input::module())
        .with_update::<(), _>(VirtualKeyboard::open)
        .updated()
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(k.is_open())));
}

#[modor_test]
fn close_opened() {
    App::new()
        .with_entity(modor_input::module())
        .with_update::<(), _>(VirtualKeyboard::open)
        .updated()
        .with_update::<(), _>(VirtualKeyboard::close)
        .updated()
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(!k.is_open())));
}

#[modor_test]
fn close_closed() {
    App::new()
        .with_entity(modor_input::module())
        .with_update::<(), _>(VirtualKeyboard::close)
        .updated()
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(!k.is_open())));
}

#[modor_test]
fn open_and_close() {
    App::new()
        .with_entity(modor_input::module())
        .with_update::<(), _>(VirtualKeyboard::open)
        .with_update::<(), _>(VirtualKeyboard::close)
        .updated()
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(k.is_open())));
}

#[modor_test]
fn close_and_open() {
    App::new()
        .with_entity(modor_input::module())
        .with_update::<(), _>(VirtualKeyboard::close)
        .with_update::<(), _>(VirtualKeyboard::open)
        .updated()
        .assert::<With<VirtualKeyboard>>(1, |e| e.has(|k: &VirtualKeyboard| assert!(k.is_open())));
}
