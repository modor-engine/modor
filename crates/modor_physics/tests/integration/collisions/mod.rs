pub mod convex_convex_2d;

use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::{App, Built, EntityMainComponent};
use modor_math::Vec3;
use modor_physics::{Collider, CollisionLayer, PhysicsModule};

enum CollisionGroup {
    MAIN,
}

impl modor_physics::CollisionGroup for CollisionGroup {
    fn index(self) -> usize {
        self as usize
    }

    fn layers() -> Vec<CollisionLayer<Self>> {
        vec![CollisionLayer::new(vec![
            CollisionGroup::MAIN,
            CollisionGroup::MAIN,
        ])]
    }
}

fn assert_collision<T, U, V, W>(
    object1: fn() -> V,
    object2: fn() -> W,
    penetration_1in2: Vec3,
    contact_centroid: Vec3,
) where
    T: EntityMainComponent,
    U: EntityMainComponent,
    V: Built<T>,
    W: Built<U>,
{
    assert_collision_internal(object1(), object2(), penetration_1in2, contact_centroid);
    assert_collision_internal(object2(), object1(), -penetration_1in2, contact_centroid);
}

fn assert_no_collision<T, U, V, W>(object1: fn() -> V, object2: fn() -> W)
where
    T: EntityMainComponent,
    U: EntityMainComponent,
    V: Built<T>,
    W: Built<U>,
{
    assert_no_collision_internal(object1(), object2());
    assert_no_collision_internal(object2(), object1());
}

fn assert_collision_internal<T, U>(
    object1: impl Built<T>,
    object2: impl Built<U>,
    penetration_1in2: Vec3,
    contact_centroid: Vec3,
) where
    T: EntityMainComponent,
    U: EntityMainComponent,
{
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build::<CollisionGroup>())
        .into();
    let object1_id = app.create_entity(object1);
    let object2_id = app.create_entity(object2);
    app.update();
    app.assert_entity(object1_id).has(|c: &Collider| {
        assert_eq!(c.collisions().len(), 1);
        assert_abs_diff_eq!(c.collisions()[0].penetration().x, penetration_1in2.x);
        assert_abs_diff_eq!(c.collisions()[0].penetration().y, penetration_1in2.y);
        assert_abs_diff_eq!(c.collisions()[0].penetration().z, penetration_1in2.z);
        assert_abs_diff_eq!(c.collisions()[0].contact_centroid().x, contact_centroid.x);
        assert_abs_diff_eq!(c.collisions()[0].contact_centroid().y, contact_centroid.y);
        assert_abs_diff_eq!(c.collisions()[0].contact_centroid().z, contact_centroid.z);
    });
    app.assert_entity(object2_id).has(|c: &Collider| {
        assert_eq!(c.collisions().len(), 1);
        assert_abs_diff_eq!(c.collisions()[0].penetration().x, -penetration_1in2.x);
        assert_abs_diff_eq!(c.collisions()[0].penetration().y, -penetration_1in2.y);
        assert_abs_diff_eq!(c.collisions()[0].penetration().z, -penetration_1in2.z);
        assert_abs_diff_eq!(c.collisions()[0].contact_centroid().x, contact_centroid.x);
        assert_abs_diff_eq!(c.collisions()[0].contact_centroid().y, contact_centroid.y);
        assert_abs_diff_eq!(c.collisions()[0].contact_centroid().z, contact_centroid.z);
    });
}

fn assert_no_collision_internal<T, U>(object1: impl Built<T>, object2: impl Built<U>)
where
    T: EntityMainComponent,
    U: EntityMainComponent,
{
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build::<CollisionGroup>())
        .into();
    let object1_id = app.create_entity(object1);
    let object2_id = app.create_entity(object2);
    app.update();
    app.assert_entity(object1_id).has(|c: &Collider| {
        assert_eq!(c.collisions().len(), 0);
    });
    app.assert_entity(object2_id).has(|c: &Collider| {
        assert_eq!(c.collisions().len(), 0);
    });
}
