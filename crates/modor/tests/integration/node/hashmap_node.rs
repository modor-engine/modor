use log::Level;
use modor::{App, Node, RootNode, Visit};
use std::collections::HashMap;

#[modor::test]
fn update_node() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    let container = app.get_mut::<Container>();
    assert!(container.0.contains(&"InnerNode(0)::on_enter".into()));
    assert!(container.0.contains(&"InnerNode(0)::on_exit".into()));
    assert!(container.0.contains(&"InnerNode(1)::on_enter".into()));
    assert!(container.0.contains(&"InnerNode(1)::on_exit".into()));
    assert!(container.0.contains(&"InnerNode(2)::on_enter".into()));
    assert!(container.0.contains(&"InnerNode(2)::on_exit".into()));
}

#[derive(Default, RootNode, Node, Visit)]
struct Container(Vec<String>);

#[derive(Node, Visit)]
struct Root(HashMap<usize, InnerNode>);

impl RootNode for Root {
    fn on_create(_app: &mut App) -> Self {
        Self(
            [(0, InnerNode(0)), (1, InnerNode(1)), (2, InnerNode(2))]
                .into_iter()
                .collect(),
        )
    }
}

#[derive(Visit)]
struct InnerNode(usize);

impl Node for InnerNode {
    fn on_enter(&mut self, app: &mut App) {
        app.get_mut::<Container>()
            .0
            .push(format!("InnerNode({})::on_enter", self.0));
    }

    fn on_exit(&mut self, app: &mut App) {
        app.get_mut::<Container>()
            .0
            .push(format!("InnerNode({})::on_exit", self.0));
    }
}
