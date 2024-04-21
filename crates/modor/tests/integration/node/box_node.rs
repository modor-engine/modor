use log::Level;
use modor::{App, Context, Node, RootNode, Visit};

#[modor::test]
fn update_node() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    let container = app.root::<Container>();
    assert_eq!(container.0, ["InnerNode::on_enter", "InnerNode::on_exit"]);
}

#[derive(Default, RootNode, Node, Visit)]
struct Container(#[modor(skip)] Vec<&'static str>);

#[derive(Node, Visit)]
struct Root(Box<dyn Node>);

impl RootNode for Root {
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self(Box::new(InnerNode))
    }
}

#[derive(Visit)]
struct InnerNode;

impl Node for InnerNode {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        ctx.root::<Container>()
            .get_mut(ctx)
            .0
            .push("InnerNode::on_enter");
    }

    fn on_exit(&mut self, ctx: &mut Context<'_>) {
        ctx.root::<Container>()
            .get_mut(ctx)
            .0
            .push("InnerNode::on_exit");
    }
}
