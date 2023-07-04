use log::LevelFilter;
use modor::{App, BuiltEntity, EntityBuilder, Not, Or, With};
use modor_internal::assert_approx_eq;

#[modor_test]
fn build_empty_entity() {
    App::new()
        .with_entity(EntityBuilder::new())
        .assert::<()>(1, |e| e.child_count(0));
}

#[modor_test]
fn build_entity_with_components() {
    App::new()
        .with_entity(
            EntityBuilder::new()
                .component(Integer(10))
                .component(Text("name"))
                .with(|s| s.0 = "new name")
                .component(Integer(20)),
        )
        .assert::<()>(1, |e| {
            e.has(|i: &Integer| assert_eq!(i.0, 20))
                .has(|t: &Text| assert_eq!(t.0, "new name"))
                .child_count(0)
        });
}

#[modor_test]
fn build_entity_with_option_component() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(
            EntityBuilder::new()
                .component(Integer(10))
                .component_option::<Integer>(None)
                .component_option::<Float>(None)
                .with(|s| s.0 = 2.)
                .component_option(Some(Text("name")))
                .with(|s| s.0 = "new name"),
        )
        .assert::<()>(1, |e| {
            e.has(|i: &Integer| assert_eq!(i.0, 10))
                .has(|t: &Text| assert_eq!(t.0, "new name"))
                .has_not::<Float>()
                .child_count(0)
        });
}

#[modor_test]
fn build_entity_with_dependency() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(Singleton1(40))
        .with_entity(Singleton1(41))
        .with_entity(
            EntityBuilder::new()
                .dependency::<Singleton1, _, _>(|| Singleton1(10))
                .dependency::<Singleton2, _, _>(|| Singleton2(20))
                .dependency::<Singleton3, _, _>(|| Singleton3(30)),
        )
        .with_entity(Singleton3(50))
        .assert::<With<Singleton1>>(1, |e| e.has(|s: &Singleton1| assert_eq!(s.0, 41)))
        .assert::<With<Singleton2>>(1, |e| e.has(|s: &Singleton2| assert_eq!(s.0, 20)))
        .assert::<With<Singleton3>>(1, |e| e.has(|s: &Singleton3| assert_eq!(s.0, 50)))
        .assert::<Not<Or<(With<Singleton1>, With<Singleton2>, With<Singleton3>)>>>(1, |e| {
            e.has_not::<Singleton1>()
                .has_not::<Singleton2>()
                .has_not::<Singleton3>()
        });
}

#[modor_test]
fn build_entity_with_child_component() {
    App::new()
        .with_entity(
            EntityBuilder::new()
                .component(Singleton1(1))
                .child_component(Singleton2(4))
                .child_component(Singleton2(2))
                .with(|s| s.0 += 1)
                .child_component(Singleton3(3)),
        )
        .assert::<With<Singleton1>>(1, |e| e.child_count(2))
        .assert::<With<Singleton2>>(1, |e| {
            e.child_count(0)
                .has_parent::<With<Singleton1>>()
                .has(|s: &Singleton2| assert_eq!(s.0, 3))
        })
        .assert::<With<Singleton3>>(1, |e| e.child_count(0).has_parent::<With<Singleton1>>());
}

#[modor_test]
fn build_entity_with_child_entity() {
    let level3 = EntityBuilder::new().component(Singleton3(3));
    let level2 = EntityBuilder::new()
        .component(Singleton2(2))
        .child_entity(level3);
    let level1 = EntityBuilder::new()
        .component(Singleton1(1))
        .child_entity(level2);
    App::new()
        .with_entity(level1)
        .assert::<With<Singleton1>>(1, |e| e.child_count(1))
        .assert::<With<Singleton2>>(1, |e| e.child_count(1).has_parent::<With<Singleton1>>())
        .assert::<With<Singleton3>>(1, |e| e.child_count(0).has_parent::<With<Singleton2>>());
}

#[modor_test]
fn build_entity_with_child_entities() {
    App::new()
        .with_entity(
            EntityBuilder::new()
                .component(Singleton1(1))
                .child_entities(|g| {
                    g.add(Singleton2(2));
                    g.add(Singleton3(3));
                }),
        )
        .assert::<With<Singleton1>>(1, |e| e.child_count(2))
        .assert::<Or<(With<Singleton2>, With<Singleton3>)>>(2, |e| {
            e.child_count(0).has_parent::<With<Singleton1>>()
        });
}

#[modor_test]
fn build_entity_with_inherited() {
    let child = EntityBuilder::new()
        .component(Integer(10))
        .component(Float(2.))
        .component(Text("child"))
        .child_entity(Singleton3(3));
    App::new()
        .with_entity(
            EntityBuilder::new()
                .component(Integer(1))
                .component(Singleton1(1))
                .child_entity(Singleton2(2))
                .inherited(child)
                .component(Text("new child")),
        )
        .assert::<With<Singleton1>>(1, |e| {
            e.has(|i: &Integer| assert_eq!(i.0, 10))
                .has(|i: &Float| assert_approx_eq!(i.0, 2.))
                .has(|t: &Text| assert_eq!(t.0, "new child"))
                .child_count(2)
        })
        .assert::<Or<(With<Singleton2>, With<Singleton3>)>>(2, |e| {
            e.has_parent::<With<Singleton1>>()
                .has_not::<Integer>()
                .has_not::<Float>()
                .has_not::<Text>()
        });
}

#[modor_test]
fn build_entity_with_updated_component() {
    App::new()
        .with_entity(
            EntityBuilder::new()
                .component(Integer(10))
                .component_option::<Text>(None)
                .child_component(Float(2.))
                .child_entity(Singleton1(1))
                .child_entities(|_| ())
                .dependency::<Singleton2, _, _>(|| Singleton2(2))
                .inherited(EntityBuilder::new().component(Singleton3(3)))
                .updated(|i: &mut Integer| i.0 += 1),
        )
        .assert::<With<Integer>>(1, |e| e.has(|i: &Integer| assert_eq!(i.0, 11)));
}

#[derive(Component, NoSystem)]
struct Integer(u32);

#[derive(Component, NoSystem)]
struct Float(f32);

#[derive(Component, NoSystem)]
struct Text(&'static str);

#[derive(SingletonComponent, NoSystem)]
struct Singleton1(u32);

#[derive(SingletonComponent, NoSystem)]
struct Singleton2(u32);

#[derive(SingletonComponent, NoSystem)]
struct Singleton3(u32);
