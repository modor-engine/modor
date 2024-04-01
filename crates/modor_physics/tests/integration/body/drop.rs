use modor::log::Level;
use modor::{App, Node, RootNode, Visit};
use modor_math::Vec2;
use modor_physics::Body2D;

#[modor::test]
fn drop_body() {
    let mut app = App::new::<Root>(Level::Info);
    push_body(&mut app);
    push_body(&mut app);
    app.update();
    app.root::<Root>().bodies.remove(0);
    push_body(&mut app);
    app.update();
    push_body(&mut app);
    assert_eq!(app.root::<Root>().bodies[0].index().as_usize(), 1);
    assert_eq!(app.root::<Root>().bodies[1].index().as_usize(), 2);
    assert_eq!(app.root::<Root>().bodies[2].index().as_usize(), 0);
}

fn push_body(app: &mut App) {
    let body = Body2D::new(&mut app.ctx(), Vec2::ZERO, Vec2::ONE);
    app.root::<Root>().bodies.push(body);
}

#[derive(Default, RootNode, Node, Visit)]
struct Root {
    bodies: Vec<Body2D>,
}
