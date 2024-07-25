use log::Level;
use modor::{App, Node, RootNode};
use modor_graphics::CursorTracker;
use modor_input::modor_math::Vec2;
use modor_input::{Inputs, MouseButton};
use modor_internal::assert_approx_eq;

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_cursor_without_action() {
    let mut app = App::new::<Root>(Level::Info);
    let mut cursor = CursorTracker::new(&mut app);
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.666_666, 0.5));
    assert!(!cursor.state(&app).is_pressed());
    cursor.update(&mut app);
    app.update();
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.666_666, 0.5));
    assert!(!cursor.state(&app).is_pressed());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_cursor_with_mouse_action() {
    let mut app = App::new::<Root>(Level::Info);
    let mut cursor = CursorTracker::new(&mut app);
    inputs(&mut app).mouse.delta = Vec2::new(200., 100.);
    inputs(&mut app).mouse.position = Vec2::new(200., 100.);
    inputs(&mut app).mouse[MouseButton::Left].press();
    cursor.update(&mut app);
    app.update();
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.333_333, 0.333_333));
    assert!(cursor.state(&app).is_pressed());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_cursor_with_finger_pressed() {
    let mut app = App::new::<Root>(Level::Info);
    let mut cursor = CursorTracker::new(&mut app);
    inputs(&mut app).fingers[0].delta = Vec2::new(200., 100.);
    inputs(&mut app).fingers[0].position = Vec2::new(200., 100.);
    inputs(&mut app).fingers[0].state.press();
    cursor.update(&mut app);
    app.update();
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.333_333, 0.333_333));
    assert!(cursor.state(&app).is_pressed());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_cursor_with_finger_released() {
    let mut app = App::new::<Root>(Level::Info);
    let mut cursor = CursorTracker::new(&mut app);
    inputs(&mut app).fingers[0].delta = Vec2::new(200., 100.);
    inputs(&mut app).fingers[0].position = Vec2::new(200., 100.);
    inputs(&mut app).fingers[0].state.press();
    cursor.update(&mut app);
    app.update();
    inputs(&mut app).fingers[0].state.release();
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.333_333, 0.333_333));
    assert!(!cursor.state(&app).is_pressed());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_cursor_with_mouse_then_finger_action() {
    let mut app = App::new::<Root>(Level::Info);
    let mut cursor = CursorTracker::new(&mut app);
    inputs(&mut app).mouse.delta = Vec2::new(100., 50.);
    inputs(&mut app).mouse.position = Vec2::new(100., 50.);
    cursor.update(&mut app);
    app.update();
    inputs(&mut app).mouse.delta = Vec2::ZERO;
    inputs(&mut app).fingers[0].delta = Vec2::new(200., 100.);
    inputs(&mut app).fingers[0].position = Vec2::new(200., 100.);
    inputs(&mut app).fingers[0].state.press();
    cursor.update(&mut app);
    app.update();
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.333_333, 0.333_333));
    assert!(cursor.state(&app).is_pressed());
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn retrieve_cursor_with_finger_then_mouse_action() {
    let mut app = App::new::<Root>(Level::Info);
    let mut cursor = CursorTracker::new(&mut app);
    inputs(&mut app).fingers[0].delta = Vec2::new(100., 50.);
    inputs(&mut app).fingers[0].position = Vec2::new(100., 50.);
    cursor.update(&mut app);
    app.update();
    inputs(&mut app).fingers[0].delta = Vec2::ZERO;
    inputs(&mut app).mouse.delta = Vec2::new(200., 100.);
    inputs(&mut app).mouse.position = Vec2::new(200., 100.);
    inputs(&mut app).mouse[MouseButton::Left].press();
    cursor.update(&mut app);
    app.update();
    assert_approx_eq!(cursor.position(&app), Vec2::new(-0.333_333, 0.333_333));
    assert!(cursor.state(&app).is_pressed());
}

fn inputs(app: &mut App) -> &mut Inputs {
    app.get_mut::<Inputs>()
}

#[derive(Default, RootNode, Node)]
struct Root;
