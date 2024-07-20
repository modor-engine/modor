use log::Level;
use modor::{App, Node, RootNode, Visit};

#[modor::test]
fn update_node() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    let container = app.get_mut::<Container>();
    assert_eq!(
        container.0,
        [
            "TestNode::on_enter",
            "InnerNode::on_enter",
            "InnerNode::on_exit",
            "TestNode::on_exit"
        ]
    );
}

#[derive(Default, RootNode, Node, Visit)]
struct Container(Vec<&'static str>);

#[derive(Node, Visit)]
struct Root(TestNode);

impl RootNode for Root {
    fn on_create(_app: &mut App) -> Self {
        Self(TestNode::Node(InnerNode))
    }
}

#[allow(dead_code)]
#[derive(Visit)]
enum TestNode {
    Node(InnerNode),
    NotNode(usize),
    StructNode { node: InnerNode },
    StructNoNode { value: usize },
    Empty,
}

impl Node for TestNode {
    fn on_enter(&mut self, app: &mut App) {
        app.get_mut::<Container>().0.push("TestNode::on_enter");
    }

    fn on_exit(&mut self, app: &mut App) {
        app.get_mut::<Container>().0.push("TestNode::on_exit");
    }
}

#[derive(Visit)]
struct InnerNode;

impl Node for InnerNode {
    fn on_enter(&mut self, app: &mut App) {
        app.get_mut::<Container>().0.push("InnerNode::on_enter");
    }

    fn on_exit(&mut self, app: &mut App) {
        app.get_mut::<Container>().0.push("InnerNode::on_exit");
    }
}
