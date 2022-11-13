use crate::TestEntity;
use modor::{App, Built, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, DeltaTime, Dynamics2D, PhysicsModule,
    RelativeTransform2D, Transform2D,
};
use std::mem;
use std::time::Duration;

struct Entity1;
struct Entity2;
struct Entity3;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CollisionGroup {
    Group1,
    Group2,
    Group3,
}

impl CollisionGroupRef for CollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Group1, Self::Group1 | Self::Group2) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ReversedCollisionGroup {
    Group1,
    Group2,
}

impl CollisionGroupRef for ReversedCollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Group1 | Self::Group2, Self::Group1) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BigCollisionGroup {
    Group1,
    Group2,
    Group3,
    Group4,
    Group5,
    Group6,
    Group7,
    Group8,
    Group9,
    Group10,
    Group11,
    Group12,
    Group13,
    Group14,
    Group15,
    Group16,
    Group17,
    Group18,
    Group19,
    Group20,
    Group21,
    Group22,
    Group23,
    Group24,
    Group25,
    Group26,
    Group27,
    Group28,
    Group29,
    Group30,
    Group31,
    Group32,
    Group33,
}

impl CollisionGroupRef for BigCollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Group1, Self::Group32) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}

fn collider_entity1(group: impl CollisionGroupRef, with_dynamics: bool) -> impl Built<TestEntity> {
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

fn collider_entity2(group: impl CollisionGroupRef, with_dynamics: bool) -> impl Built<TestEntity> {
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
fn add_collider_with_dynamics_and_same_colliding_group() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, true))
        .with_entity(collider_entity2(CollisionGroup::Group1, true))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_same_rapier_group_but_different_modor_group() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(BigCollisionGroup::Group1, true))
        .with_entity(collider_entity2(BigCollisionGroup::Group33, true))
        .updated()
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_different_colliding_groups() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, true))
        .with_entity(collider_entity2(CollisionGroup::Group2, true))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert!(collision.has_other_entity_group(CollisionGroup::Group2));
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_same_colliding_group_with_reversed_condition() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(ReversedCollisionGroup::Group1, true))
        .with_entity(collider_entity2(ReversedCollisionGroup::Group2, true))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert!(collision.has_other_entity_group(ReversedCollisionGroup::Group2));
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert!(collision.has_other_entity_group(ReversedCollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_collider_with_dynamics_and_same_not_colliding_group() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group3, true))
        .with_entity(collider_entity2(CollisionGroup::Group3, true))
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
fn add_collider_with_dynamics_and_different_not_colliding_groups() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, true))
        .with_entity(collider_entity2(CollisionGroup::Group3, true))
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
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, false))
        .with_entity(collider_entity2(CollisionGroup::Group2, false))
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert!(collision.has_other_entity_group(CollisionGroup::Group2));
                assert_approx_eq!(collision.normal, COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert_eq!(collision.other_entity_id, entity2_id - 1);
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_collider_without_dynamics() {
    let mut collider = Collider2D::rectangle(CollisionGroup::Group2);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, false))
        .with_entity(collider_entity2(CollisionGroup::Group2, false))
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
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, true))
        .with_entity(collider_entity2(CollisionGroup::Group2, true))
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
    let mut collider = Collider2D::rectangle(CollisionGroup::Group2);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, true))
        .with_entity(collider_entity2(CollisionGroup::Group2, true))
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
        .with(Collider2D::rectangle(CollisionGroup::Group3));
    let mut collider = Collider2D::rectangle(CollisionGroup::Group3);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(collider_entity1(CollisionGroup::Group1, false))
        .with_entity(collider_entity2(CollisionGroup::Group2, false))
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
