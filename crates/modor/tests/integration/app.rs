use log::Level;
use modor::{App, RootNode};

#[modor::test]
fn create_node() {
    let mut app = App::new::<Root>(Level::Info);
    app.create::<Root>();
    assert_eq!(app.get_mut::<Counter>().value, 1);
}

#[modor::test]
fn create_node_handle() {
    let mut app = App::new::<Root>(Level::Info);
    let handle = app.handle::<Root>();
    assert_eq!(app.get_mut::<Counter>().value, 1);
    assert_eq!(handle.get(&app).value, 0);
    assert_eq!(handle.get_mut(&mut app).value, 0);
}

#[modor::test]
fn take_node() {
    let mut app = App::new::<Root>(Level::Info);
    let result = app.take(|root: &mut Root, app| {
        assert_eq!(app.get_mut::<Counter>().value, 1);
        assert_eq!(root.value, 0);
        42
    });
    assert_eq!(result, 42);
}

#[modor::test]
fn take_node_handle() {
    let mut app = App::new::<Root>(Level::Info);
    let result = app.handle::<Root>().take(&mut app, |root: &mut Root, app| {
        assert_eq!(app.get_mut::<Counter>().value, 1);
        assert_eq!(root.value, 0);
        42
    });
    assert_eq!(result, 42);
}

struct Root {
    value: usize,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        app.get_mut::<Counter>().value += 1;
        Self { value: 0 }
    }
}

#[derive(Default, RootNode)]
struct Counter {
    value: usize,
}
