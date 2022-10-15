use crate::TestEntity;
use modor::{App, Built, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupIndex, CollisionLayer, DeltaTime, Dynamics2D, PhysicsModule,
    RelativeTransform2D, Transform2D,
};
use std::mem;
use std::time::Duration;

struct Entity1;
struct Entity2;
struct Entity3;

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
        .with(Entity1)
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
        .with(Entity2)
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
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .with_entity(collider_entity2(CollisionGroup::GROUP1, true))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                let group = CollisionGroup::GROUP1.into();
                assert_eq!(collision.other_entity_group, group);
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                let group = CollisionGroup::GROUP1.into();
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert_eq!(collision.other_entity_group, group);
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_different_groups_in_same_layer() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .with_entity(collider_entity2(CollisionGroup::GROUP2, true))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                let group = CollisionGroup::GROUP2.into();
                assert_eq!(collision.other_entity_group, group);
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                let group = CollisionGroup::GROUP1.into();
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert_eq!(collision.other_entity_group, group);
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_group_not_duplicated_in_layer() {
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP3, true))
        .with_entity(collider_entity2(CollisionGroup::GROUP3, true))
        .updated()
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
                .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_different_groups_not_in_same_layer() {
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .with_entity(collider_entity2(CollisionGroup::GROUP3, true))
        .updated()
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
                .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_without_dynamics() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, false))
        .with_entity(collider_entity2(CollisionGroup::GROUP2, false))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                let group = CollisionGroup::GROUP2.into();
                assert_eq!(collision.other_entity_group, group);
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                let group = CollisionGroup::GROUP1.into();
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert_eq!(collision.other_entity_group, group);
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_collider_without_dynamics() {
    let mut collider = Collider2D::rectangle(CollisionGroup::GROUP2);
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, false))
        .with_entity(collider_entity2(CollisionGroup::GROUP2, false))
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1))
        })
        .with_update::<With<Entity2>, _>(|c: &mut Collider2D| mem::swap(c, &mut collider))
        .updated()
        .with_update::<With<Entity2>, _>(|t: &mut Transform2D| *t.position = Vec2::new(1., 2.))
        .updated()
        .with_update::<With<Entity2>, _>(|c: &mut Collider2D| mem::swap(c, &mut collider))
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_dynamics_with_collider() {
    let mut dynamics = Dynamics2D::new();
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .with_entity(collider_entity2(CollisionGroup::GROUP2, true))
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1))
        })
        .with_update::<With<Entity2>, _>(|d: &mut Dynamics2D| mem::swap(d, &mut dynamics))
        .updated()
        .with_update::<With<Entity2>, _>(|d: &mut Dynamics2D| mem::swap(d, &mut dynamics))
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_collider_with_dynamics() {
    let mut collider = Collider2D::rectangle(CollisionGroup::GROUP2);
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, true))
        .with_entity(collider_entity2(CollisionGroup::GROUP2, true))
        .updated()
        .with_update::<With<Entity2>, _>(|d: &mut Dynamics2D| *d.velocity = Vec2::new(1., 0.))
        .updated()
        .with_update::<With<Entity2>, _>(|c: &mut Collider2D| mem::swap(c, &mut collider))
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 2.)))
        })
        .with_update::<With<Entity2>, _>(|c: &mut Collider2D| mem::swap(c, &mut collider))
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(5., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn move_collider() {
    let entity3 = EntityBuilder::new(TestEntity)
        .with(Entity3)
        .with(
            Transform2D::new()
                .with_position(Vec2::new(-5., 2.))
                .with_size(Vec2::ONE),
        )
        .with(Collider2D::rectangle(CollisionGroup::GROUP3));
    let mut collider = Collider2D::rectangle(CollisionGroup::GROUP3);
    App::new()
        .with_entity(PhysicsModule::build_with_layers(layers()))
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::GROUP1, false))
        .with_entity(collider_entity2(CollisionGroup::GROUP2, false))
        .with_entity(entity3)
        .updated()
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 1))
        })
        .with_update::<With<Entity2>, _>(|c: &mut Collider2D| mem::swap(c, &mut collider))
        .with_update::<With<Entity3>, _>(|c: &mut Collider2D| mem::swap(c, &mut collider))
        .updated()
        .assert::<With<Entity3>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        });
}
