use log::Level;
use modor::{App, Context, Node, RootNode, Visit};

#[modor::test]
fn update_node_with_inner_node() {
    let mut app = App::new::<Root<true>>(Level::Info);
    app.update();
    let container = app.root::<Container>();
    assert_eq!(container.0, ["InnerNode::on_enter", "InnerNode::on_exit"]);
}

#[modor::test]
fn update_node_without_inner_node() {
    let mut app = App::new::<Root<false>>(Level::Info);
    app.update();
    let container = app.root::<Container>();
    assert!(container.0.is_empty());
}

#[derive(Default, RootNode, Node, Visit)]
struct Container(Vec<&'static str>);

#[derive(Node, Visit)]
struct Root<const IS_SOME: bool>(Option<InnerNode>);

impl<const IS_SOME: bool> RootNode for Root<IS_SOME> {
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self(IS_SOME.then_some(InnerNode))
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
