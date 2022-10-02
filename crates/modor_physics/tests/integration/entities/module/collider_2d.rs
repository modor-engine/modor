use crate::entities::module::{Action, MoveDst, TestEntity, Updates, DELTA_TIME};
use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupIndex, CollisionLayer, DeltaTime, Dynamics2D, PhysicsModule,
    RelativeTransform2D, Transform2D,
};

enum CollisionGroup {
    GROUP1,
    GROUP2,
    GROUP3,
}

impl From<CollisionGroup> for CollisionGroupIndex {
    fn from(group: CollisionGroup) -> Self {
        match group {
            CollisionGroup::GROUP1 => Self::Group0,
            CollisionGroup::GROUP2 => Self::Group1,
            CollisionGroup::GROUP3 => Self::Group2,
        }
    }
}

fn layers() -> Vec<CollisionLayer> {
    vec![
        CollisionLayer::new(vec![
            CollisionGroup::GROUP1.into(),
            CollisionGroup::GROUP1.into(),
        ]),
        CollisionLayer::new(vec![
            CollisionGroup::GROUP2.into(),
            CollisionGroup::GROUP1.into(),
        ]),
    ]
}

fn collider_entity1(group: CollisionGroup, with_dynamics: bool) -> impl Built<TestEntity> {
    EntityBuilder::new(TestEntity)
        .with(
            Transform2D::new()
                .with_position(Vec2::new(-2., 2.))
                .with_size(Vec2::ONE * 2.),
        )
        .with(RelativeTransform2D::new()) // make sure it has no impact on the collider
        .with_option(with_dynamics.then(Dynamics2D::new))
        .with(Collider2D::rectangle(group))
}

fn collider_entity2(group: CollisionGroup, with_dynamics: bool) -> impl Built<TestEntity> {
    EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(
            Transform2D::new()
                .with_position(Vec2::new(-1., 2.))
                .with_size(Vec2::ONE),
        )
        .with_option(with_dynamics.then(Dynamics2D::new))
        .with(Collider2D::rectangle(group))
}

const COLLISION_NORMAL: Vec2 = Vec2::X;
const COLLISION_POSITION1: Vec2 = Vec2::new(-1., 2.);
const COLLISION_POSITION2: Vec2 = Vec2::new(-1.5, 2.);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_group_duplicated_in_layer() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity1_id = app.create_entity(collider_entity1(CollisionGroup::GROUP1, true));
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP1, true));
    app.update();
    app.assert_entity(entity1_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity2_id);
        let group = CollisionGroup::GROUP1.into();
        assert_eq!(collision.other_entity_group, group);
        assert_approx_eq!(collision.normal, COLLISION_NORMAL);
        assert_approx_eq!(collision.position, COLLISION_POSITION1);
    });
    app.assert_entity(entity2_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity1_id);
        let group = CollisionGroup::GROUP1.into();
        assert_eq!(collision.other_entity_group, group);
        assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
        assert_approx_eq!(collision.position, COLLISION_POSITION2);
    });
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_different_groups_in_same_layer() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity1_id = app.create_entity(collider_entity1(CollisionGroup::GROUP1, true));
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP2, true));
    app.update();
    app.assert_entity(entity1_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity2_id);
        let group = CollisionGroup::GROUP2.into();
        assert_eq!(collision.other_entity_group, group);
        assert_approx_eq!(collision.normal, COLLISION_NORMAL);
        assert_approx_eq!(collision.position, COLLISION_POSITION1);
    });
    app.assert_entity(entity2_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity1_id);
        let group = CollisionGroup::GROUP1.into();
        assert_eq!(collision.other_entity_group, group);
        assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
        assert_approx_eq!(collision.position, COLLISION_POSITION2);
    });
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_group_not_duplicated_in_layer() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity1_id = app.create_entity(collider_entity1(CollisionGroup::GROUP3, true));
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP3, true));
    app.update();
    app.assert_entity(entity1_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_different_groups_not_in_same_layer() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity1_id = app.create_entity(collider_entity1(CollisionGroup::GROUP1, true));
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP3, true));
    app.update();
    app.assert_entity(entity1_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_without_dynamics() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity1_id = app.create_entity(collider_entity1(CollisionGroup::GROUP1, false));
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP2, false));
    app.update();
    app.assert_entity(entity1_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity2_id);
        let group = CollisionGroup::GROUP2.into();
        assert_eq!(collision.other_entity_group, group);
        assert_approx_eq!(collision.normal, COLLISION_NORMAL);
        assert_approx_eq!(collision.position, COLLISION_POSITION1);
    });
    app.assert_entity(entity2_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity1_id);
        let group = CollisionGroup::GROUP1.into();
        assert_eq!(collision.other_entity_group, group);
        assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
        assert_approx_eq!(collision.position, COLLISION_POSITION2);
    });
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_collider_without_dynamics() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, false))
        .into();
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP2, false));
    app.update();
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::RemoveCollider));
    app.update();
    app.update();
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetPosition(Vec2::new(1., 2.))));
    app.update();
    app.update();
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::PutBackCollider));
    app.update();
    app.update();
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_dynamics_with_collider() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .into();
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP2, true));
    app.update();
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::RemoveDynamics));
    app.update();
    app.update();
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::PutBackDynamics));
    app.update();
    app.update();
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_collider_with_dynamics() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .into();
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP2, true));
    app.update();
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetVelocity(Vec2::new(1., 0.))));
    app.update();
    app.update();
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::RemoveCollider));
    app.update();
    app.update();
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(5., 2.)));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::PutBackCollider));
    app.update();
    app.update();
    app.assert_entity(entity2_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(9., 2.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn move_collider() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(DELTA_TIME))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, false))
        .into();
    let entity3 = EntityBuilder::new(TestEntity)
        .with(
            Transform2D::new()
                .with_position(Vec2::new(-5., 2.))
                .with_size(Vec2::ONE),
        )
        .with(MoveDst);
    let entity2_id = app.create_entity(collider_entity2(CollisionGroup::GROUP2, false));
    let entity3_id = app.create_entity(entity3);
    app.update();
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::MoveCollider));
    app.update();
    app.update();
    app.assert_entity(entity3_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
}
