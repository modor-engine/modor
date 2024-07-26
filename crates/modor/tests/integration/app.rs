use log::Level;
use modor::{App, FromApp, State};

#[modor::test]
fn create_state() {
    let mut app = App::new::<Root>(Level::Info);
    app.create::<Root>();
    assert_eq!(app.get_mut::<Counter>().value, 1);
}

#[modor::test]
fn create_state_handle() {
    let mut app = App::new::<Root>(Level::Info);
    let handle = app.handle::<Root>();
    assert_eq!(app.get_mut::<Counter>().value, 1);
    assert_eq!(handle.get(&app).value, 42);
    assert_eq!(handle.get_mut(&mut app).value, 42);
}

#[modor::test]
fn take_state() {
    let mut app = App::new::<Root>(Level::Info);
    let result = app.take(|root: &mut Root, app| {
        assert_eq!(app.get_mut::<Counter>().value, 1);
        assert_eq!(root.value, 42);
        42
    });
    assert_eq!(result, 42);
}

#[modor::test]
fn take_state_handle() {
    let mut app = App::new::<Root>(Level::Info);
    let result = app.handle::<Root>().take(&mut app, |root: &mut Root, app| {
        assert_eq!(app.get_mut::<Counter>().value, 1);
        assert_eq!(root.value, 42);
        42
    });
    assert_eq!(result, 42);
}

struct Root {
    value: usize,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        app.get_mut::<Counter>().value += 1;
        Self { value: 0 }
    }
}

impl State for Root {
    fn init(&mut self, _app: &mut App) {
        self.value = 42;
    }
}

#[derive(Default, State)]
struct Counter {
    value: usize,
}
