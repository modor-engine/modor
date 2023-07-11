use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{DeltaTime, PhysicsModule, RelativeTransform2D, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::time::Duration;

#[derive(Component, NoSystem)]
struct RootEntity;

#[derive(Component, NoSystem)]
struct RelativeChild;

#[derive(Component, NoSystem)]
struct AbsoluteChild;

#[modor_test]
fn update_relative_position() {
    let relative_child = EntityBuilder::new()
        .component(RelativeChild)
        .component(Transform2D::new())
        .with(|t| *t.position = Vec2::new(0.1, 0.2))
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::new(0.5, 0.2)));
    let absolute_child = EntityBuilder::new()
        .component(AbsoluteChild)
        .component(Transform2D::new())
        .with(|t| *t.position = Vec2::new(0.3, 0.4))
        .component(RelativeTransform2D::new());
    let root = EntityBuilder::new()
        .component(RootEntity)
        .component(Transform2D::new())
        .with(|t| *t.position = Vec2::new(1., 2.))
        .with(|t| *t.size = Vec2::new(2., 4.))
        .with(|t| *t.rotation = FRAC_PI_2)
        .component(RelativeTransform2D::new())
        .with(|t| t.position = Some(Vec2::new(3., 4.)))
        .child_entity(relative_child)
        .child_entity(absolute_child);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(root)
        .updated()
        .assert::<With<RootEntity>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 4.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.position.unwrap(), Vec2::new(3., 4.));
                })
        })
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(2.2, 5.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.position.unwrap(), Vec2::new(0.5, 0.2));
                })
        })
        .assert::<With<AbsoluteChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(0.3, 0.4)))
        })
        .with_update::<With<RelativeChild>, _>(|t: &mut RelativeTransform2D| {
            t.position = Some(Vec2::new(10., 20.));
        })
        .updated()
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-77., 24.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.position.unwrap(), Vec2::new(10., 20.));
                })
        })
        .with_update::<With<RelativeChild>, _>(|t: &mut Transform2D| {
            *t.position = Vec2::new(5., 10.);
        })
        .updated()
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-77., 24.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.position.unwrap(), Vec2::new(10., 20.));
                })
        });
}

#[modor_test]
fn update_relative_size() {
    let relative_child = EntityBuilder::new()
        .component(RelativeChild)
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(2., 4.))
        .component(RelativeTransform2D::new())
        .with(|t| t.size = Some(Vec2::new(0.5, 0.2)));
    let absolute_child = EntityBuilder::new()
        .component(AbsoluteChild)
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(5., 10.))
        .component(RelativeTransform2D::new());
    let root = EntityBuilder::new()
        .component(RootEntity)
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(2., 4.))
        .component(RelativeTransform2D::new())
        .with(|t| t.size = Some(Vec2::new(3., 5.)))
        .child_entity(relative_child)
        .child_entity(absolute_child);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(root)
        .updated()
        .assert::<With<RootEntity>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(3., 5.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.size.unwrap(), Vec2::new(3., 5.));
                })
        })
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(1.5, 1.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.size.unwrap(), Vec2::new(0.5, 0.2));
                })
        })
        .assert::<With<AbsoluteChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(5., 10.)))
        })
        .with_update::<With<RelativeChild>, _>(|t: &mut RelativeTransform2D| {
            t.size = Some(Vec2::new(10., 20.));
        })
        .updated()
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(30., 100.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.size.unwrap(), Vec2::new(10., 20.));
                })
        })
        .with_update::<With<RelativeChild>, _>(|t: &mut Transform2D| *t.size = Vec2::new(6., 30.))
        .updated()
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(30., 100.)))
                .has(|t: &RelativeTransform2D| {
                    assert_approx_eq!(t.size.unwrap(), Vec2::new(10., 20.));
                })
        });
}

#[modor_test]
fn update_relative_rotation() {
    let relative_child = EntityBuilder::new()
        .component(RelativeChild)
        .component(Transform2D::new())
        .with(|t| *t.rotation = 0.)
        .component(RelativeTransform2D::new())
        .with(|t| t.rotation = Some(FRAC_PI_2));
    let absolute_child = EntityBuilder::new()
        .component(AbsoluteChild)
        .component(Transform2D::new())
        .with(|t| *t.rotation = FRAC_PI_4)
        .component(RelativeTransform2D::new());
    let root = EntityBuilder::new()
        .component(RootEntity)
        .component(Transform2D::new())
        .with(|t| *t.rotation = 0.)
        .component(RelativeTransform2D::new())
        .with(|t| t.rotation = Some(PI))
        .child_entity(relative_child)
        .child_entity(absolute_child);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(root)
        .updated()
        .assert::<With<RootEntity>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, PI))
                .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), PI))
        })
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 3. * FRAC_PI_2))
                .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), FRAC_PI_2))
        })
        .assert::<With<AbsoluteChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, FRAC_PI_4))
        })
        .with_update::<With<RelativeChild>, _>(|t: &mut RelativeTransform2D| t.rotation = Some(PI))
        .updated()
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 2. * PI))
                .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), PI))
        })
        .with_update::<With<RelativeChild>, _>(|t: &mut Transform2D| *t.rotation = FRAC_PI_2)
        .updated()
        .assert::<With<RelativeChild>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 2. * PI))
                .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), PI))
        });
}
