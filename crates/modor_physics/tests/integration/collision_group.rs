use std::time::Duration;
use modor::log::Level;
use modor::{App, Context, Node, RootNode, Visit};
use modor_math::Vec2;
use modor_physics::{Body2D, CollisionGroup, CollisionType, Delta};

#[modor::test]
fn recreate_group() {
    let mut app = App::new::<Root>(Level::Info);
    let group1 = CollisionGroup::new(&mut app.ctx());
    let group2 = CollisionGroup::new(&mut app.ctx());
    drop(group2);
    let group3 = CollisionGroup::new(&mut app.ctx());
    assert_eq!(group1.index(), 0);
    assert_eq!(group3.index(), 2);
    app.update();
    let group4 = CollisionGroup::new(&mut app.ctx());
    assert_eq!(group4.index(), 1);
}

#[modor::test]
fn drop_group_and_associated_interactions() {
    let mut app = App::new::<Root>(Level::Info);
    let group1 = CollisionGroup::new(&mut app.ctx());
    let group2 = CollisionGroup::new(&mut app.ctx());
    let group2_index = group2.index();
    group1.add_interaction(&mut app.ctx(), &group2, CollisionType::Sensor);
    body1(&mut app).collision_group = Some(group1);
    body2(&mut app).collision_group = Some(group2);
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 1);
    assert_eq!(body2(&mut app).collisions().len(), 1);
    body2(&mut app).collision_group = None;
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 0);
    assert_eq!(body2(&mut app).collisions().len(), 0);
    let group2 = CollisionGroup::new(&mut app.ctx());
    assert_eq!(group2.index(), group2_index);
    body2(&mut app).collision_group = Some(group2);
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 0);
    assert_eq!(body2(&mut app).collisions().len(), 0);
}

fn body1(app: &mut App) -> &mut Body2D {
    &mut app.root::<Root>().body1
}

fn body2(app: &mut App) -> &mut Body2D {
    &mut app.root::<Root>().body2
}

#[derive(Node, Visit)]
struct Root {
    body1: Body2D,
    body2: Body2D,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.root::<Delta>().duration = Duration::from_secs(2);
        Self {
            body1: Body2D::new(ctx, Vec2::ZERO, Vec2::ONE),
            body2: Body2D::new(ctx, Vec2::ZERO, Vec2::ONE),
        }
    }
}
