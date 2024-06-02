use log::Level;
use modor::{App, Const, Context, Node, RootNode, Visit};

#[modor::test]
fn update_node() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    app.update();
    app.update();
    let container = app.get_mut::<Container>();
    assert_eq!(container.0, ["InnerNode::on_enter", "InnerNode::on_exit"]);
}

#[derive(Default, RootNode, Node, Visit)]
struct Container(Vec<&'static str>);

#[derive(Node, Visit)]
struct Root(Const<InnerNode>);

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let constant = InnerNode(42).into_const(ctx);
        assert_eq!(constant.0, 42);
        Self(constant)
    }
}

#[derive(Visit)]
struct InnerNode(u32);

impl Node for InnerNode {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        ctx.get_mut::<Container>().0.push("InnerNode::on_enter");
    }

    fn on_exit(&mut self, ctx: &mut Context<'_>) {
        ctx.get_mut::<Container>().0.push("InnerNode::on_exit");
    }
}
