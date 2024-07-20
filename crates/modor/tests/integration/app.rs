use log::Level;
use modor::{App, Node, RootNode, Visit};

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

#[derive(Node, Visit)]
struct Root {
    value: usize,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        app.get_mut::<Counter>().value += 1;
        Self { value: 0 }
    }
}

#[derive(Default, RootNode, Node, Visit)]
struct Counter {
    value: usize,
}
