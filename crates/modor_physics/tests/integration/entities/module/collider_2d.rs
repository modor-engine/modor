use log::LevelFilter;
use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, DeltaTime, Dynamics2D, PhysicsModule,
    RelativeTransform2D, Transform2D,
};
use std::mem;
use std::time::Duration;

#[derive(Component, NoSystem)]
struct Entity1;

#[derive(Component, NoSystem)]
struct Entity2;

#[derive(Component, NoSystem)]
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
struct BigCollisionGroup(u32);

impl CollisionGroupRef for BigCollisionGroup {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self(0), Self(0)) | (Self(32), Self(32)) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}

fn collider_entity1(group: impl CollisionGroupRef, with_dynamics: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Entity1)
        .component(
            Transform2D::new()
                .with_position(Vec2::new(-2., 2.))
                .with_size(Vec2::ONE * 2.),
        )
        .component(RelativeTransform2D::new()) // make sure it has no impact on the collider
        .component_option(with_dynamics.then(Dynamics2D::new))
        .component(Collider2D::rectangle(group))
}

fn collider_entity2(group: impl CollisionGroupRef, with_dynamics: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Entity2)
        .component(
            Transform2D::new()
                .with_position(Vec2::new(-1., 2.))
                .with_size(Vec2::ONE),
        )
        .component_option(with_dynamics.then(Dynamics2D::new))
        .component(Collider2D::rectangle(group))
}

const COLLISION_NORMAL: Vec2 = Vec2::X;
const COLLISION_POSITION1: Vec2 = Vec2::new(-1., 2.);
const COLLISION_POSITION2: Vec2 = Vec2::new(-1.5, 2.);

#[modor_test]
fn add_collider_with_dynamics_and_same_colliding_group() {
    let mut entity2_id = 0;
    App::new()
        .with_log_level(LevelFilter::Debug)
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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
                assert_eq!(collision.other_entity_id, entity2_id - 2);
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[modor_test]
fn add_collider_with_dynamics_and_same_rapier_group_but_different_modor_group() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(collider_entity1(BigCollisionGroup(0), true))
        .with_entity(collider_entity1(BigCollisionGroup(1), true))
        .with_entity(collider_entity1(BigCollisionGroup(2), true))
        .with_entity(collider_entity1(BigCollisionGroup(3), true))
        .with_entity(collider_entity1(BigCollisionGroup(4), true))
        .with_entity(collider_entity1(BigCollisionGroup(5), true))
        .with_entity(collider_entity1(BigCollisionGroup(6), true))
        .with_entity(collider_entity1(BigCollisionGroup(7), true))
        .with_entity(collider_entity1(BigCollisionGroup(8), true))
        .with_entity(collider_entity1(BigCollisionGroup(9), true))
        .with_entity(collider_entity1(BigCollisionGroup(10), true))
        .with_entity(collider_entity1(BigCollisionGroup(11), true))
        .with_entity(collider_entity1(BigCollisionGroup(12), true))
        .with_entity(collider_entity1(BigCollisionGroup(13), true))
        .with_entity(collider_entity1(BigCollisionGroup(14), true))
        .with_entity(collider_entity1(BigCollisionGroup(15), true))
        .with_entity(collider_entity1(BigCollisionGroup(16), true))
        .with_entity(collider_entity1(BigCollisionGroup(17), true))
        .with_entity(collider_entity1(BigCollisionGroup(18), true))
        .with_entity(collider_entity1(BigCollisionGroup(19), true))
        .with_entity(collider_entity1(BigCollisionGroup(20), true))
        .with_entity(collider_entity1(BigCollisionGroup(21), true))
        .with_entity(collider_entity1(BigCollisionGroup(22), true))
        .with_entity(collider_entity1(BigCollisionGroup(23), true))
        .with_entity(collider_entity1(BigCollisionGroup(24), true))
        .with_entity(collider_entity1(BigCollisionGroup(25), true))
        .with_entity(collider_entity1(BigCollisionGroup(26), true))
        .with_entity(collider_entity1(BigCollisionGroup(27), true))
        .with_entity(collider_entity1(BigCollisionGroup(28), true))
        .with_entity(collider_entity1(BigCollisionGroup(29), true))
        .with_entity(collider_entity1(BigCollisionGroup(30), true))
        .with_entity(collider_entity1(BigCollisionGroup(31), true))
        .with_entity(collider_entity2(BigCollisionGroup(32), true))
        .updated()
        .assert::<With<Entity1>>(32, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        });
}

#[modor_test]
fn add_collider_with_dynamics_and_different_colliding_groups() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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
                assert_eq!(collision.other_entity_id, entity2_id - 2);
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[modor_test]
fn add_collider_with_dynamics_and_same_colliding_group_with_reversed_condition() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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
                assert_eq!(collision.other_entity_id, entity2_id - 2);
                assert!(collision.has_other_entity_group(ReversedCollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[modor_test]
fn add_collider_with_dynamics_and_same_not_colliding_group() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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

#[modor_test]
fn add_collider_with_dynamics_and_different_not_colliding_groups() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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

#[modor_test]
fn add_collider_without_dynamics() {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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
                assert_eq!(collision.other_entity_id, entity2_id - 2);
                assert!(collision.has_other_entity_group(CollisionGroup::Group1));
                assert_approx_eq!(collision.normal, -COLLISION_NORMAL);
                assert_approx_eq!(collision.position, COLLISION_POSITION2);
            })
            .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-1., 2.)))
        });
}

#[modor_test]
fn remove_and_put_back_collider_without_dynamics() {
    let mut collider = Collider2D::rectangle(CollisionGroup::Group2);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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

#[modor_test]
fn remove_and_put_back_dynamics_with_collider() {
    let mut dynamics = Dynamics2D::new();
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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

#[modor_test]
fn remove_and_put_back_collider_with_dynamics() {
    let mut collider = Collider2D::rectangle(CollisionGroup::Group2);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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

#[modor_test]
fn move_collider() {
    let entity3 = EntityBuilder::new()
        .component(Entity3)
        .component(
            Transform2D::new()
                .with_position(Vec2::new(-5., 2.))
                .with_size(Vec2::ONE),
        )
        .component(Collider2D::rectangle(CollisionGroup::Group3));
    let mut collider = Collider2D::rectangle(CollisionGroup::Group3);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
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
