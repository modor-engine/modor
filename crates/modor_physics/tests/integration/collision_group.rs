use modor::log::Level;
use modor::{App, Context, Node, RootNode, Visit};
use modor_physics::{Body2D, CollisionGroup, CollisionType};

#[modor::test]
fn drop_group() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    app.update();
    assert_eq!(body2(&mut app).collisions().len(), 1);
    body2(&mut app).collision_group = None;
    *group2(&mut app) = None;
    app.update();
    create_group2_with_no_interaction(&mut app);
    app.update();
    app.update();
    assert_eq!(body2(&mut app).collisions().len(), 0);
}

fn create_group2_with_no_interaction(app: &mut App) {
    let collision_group = CollisionGroup::new(&mut app.ctx());
    body2(app).collision_group = Some(collision_group.glob().clone());
    *group2(app) = Some(collision_group);
}

fn body2(app: &mut App) -> &mut Body2D {
    &mut app.get_mut::<Root>().body2
}

fn group2(app: &mut App) -> &mut Option<CollisionGroup> {
    &mut app.get_mut::<Root>().group2
}

#[derive(Node, Visit)]
struct Root {
    group1: CollisionGroup,
    group2: Option<CollisionGroup>,
    body1: Body2D,
    body2: Body2D,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let group1 = CollisionGroup::new(ctx);
        let group2 = CollisionGroup::new(ctx);
        group1.add_interaction(ctx, group2.glob(), CollisionType::Sensor);
        let body1 = Body2D::new(ctx).with_collision_group(Some(group1.glob().clone()));
        let body2 = Body2D::new(ctx).with_collision_group(Some(group2.glob().clone()));
        Self {
            group1,
            group2: Some(group2),
            body1,
            body2,
        }
    }
}
