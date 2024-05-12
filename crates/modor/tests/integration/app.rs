use log::Level;
use modor::App;
use modor_derive::{Node, RootNode, Visit};

#[modor::test]
fn create_node_handle() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let handle = ctx.root::<Root>();
    assert_eq!(handle.get(&ctx).value, 0);
    assert_eq!(handle.get_mut(&mut ctx).value, 0);
}

#[derive(Default, RootNode, Node, Visit)]
struct Root {
    value: usize,
}
