use log::Level;
use modor::{App, Context, Node, RootNode, Visit};

#[modor::test]
fn create_node() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    ctx.create::<Root>();
    assert_eq!(app.get_mut::<Counter>().value, 1);
}

#[modor::test]
fn create_node_handle() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let handle = ctx.handle::<Root>();
    assert_eq!(app.get_mut::<Counter>().value, 1);
    assert_eq!(handle.get(&app.ctx()).value, 0);
    assert_eq!(handle.get_mut(&mut app.ctx()).value, 0);
}

#[derive(Node, Visit)]
struct Root {
    value: usize,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.get_mut::<Counter>().value += 1;
        Self { value: 0 }
    }
}

#[derive(Default, RootNode, Node, Visit)]
struct Counter {
    value: usize,
}
