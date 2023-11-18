use modor::{App, BuiltEntity, EntityBuilder, Not, With};

#[modor_test(disabled(wasm))]
fn create_app_with_thread_count_and_log_level() {
    let app = App::new().with_thread_count(2);
    assert_eq!(app.thread_count(), 2);
    let app = app.with_thread_count(1);
    assert_eq!(app.thread_count(), 1);
    let app = app.with_thread_count(0);
    assert_eq!(app.thread_count(), 1);
}

#[modor_test(disabled(windows, linux, macos, android))]
fn create_app_with_thread_count_and_log_level_for_wasm() {
    let app = App::new().with_thread_count(2);
    assert_eq!(app.thread_count(), 1);
}

#[modor_test]
fn assert_valid_entity_count() {
    let composed_entity = EntityBuilder::new()
        .component(Component1(30))
        .component(Component2(40));
    App::new()
        .with_entity(Component1(10))
        .with_entity(Component2(20))
        .with_entity(composed_entity)
        .with_entity(Singleton1(50))
        .assert::<With<Component1>>(2, |e| e)
        .assert::<With<Component2>>(2, |e| e)
        .assert::<With<Singleton1>>(1, |e| e)
        .assert::<With<Singleton2>>(0, |e| e)
        .assert::<(With<Component1>, With<Component2>)>(1, |e| e)
        .assert::<(With<Singleton1>, With<Component1>)>(0, |e| e);
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: 2 entities matching \
modor::filters::with::With<integration::app::Component1>, actual count: 1"]
fn assert_invalid_entity_count() {
    App::new()
        .with_entity(Component1(10))
        .assert::<With<Component1>>(2, |e| e);
}

#[modor_test(disabled(wasm))]
fn assert_entity_has_existing_component() {
    App::new()
        .with_entity(Component1(10))
        .with_entity(Component1(20))
        .with_entity(Singleton1(30))
        .assert::<With<Component1>>(2, |e| {
            e.has(|c: &Component1| assert!(c.0 == 10 || c.0 == 20))
        })
        .assert_any::<With<Component1>>(2, |e| {
            e.has(|c: &Component1| assert_eq!(c.0, 10))
                .has(|c: &Component1| assert_eq!(c.0, 20))
                .has(|c: &Component1| assert!(c.0 > 0))
        })
        .assert::<With<Singleton1>>(1, |e| e.has(|c: &Singleton1| assert_eq!(c.0, 30)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion `left == right` failed"]
fn assert_entity_has_invalid_component() {
    App::new()
        .with_entity(Component1(10))
        .assert::<With<Component1>>(1, |e| e.has(|c: &Component1| assert_eq!(c.0, 20)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion `left == right` failed"]
fn assert_entity_has_invalid_component_in_any_mode() {
    App::new()
        .with_entity(Component1(10))
        .assert_any::<With<Component1>>(1, |e| e.has(|c: &Component1| assert_eq!(c.0, 20)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> \
have component integration::app::Component2"]
fn assert_entity_has_missing_component() {
    App::new()
        .with_entity(Component1(10))
        .assert::<With<Component1>>(1, |e| e.has(|_: &Component2| ()));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> \
have component integration::app::Component2"]
fn assert_entity_has_missing_component_in_any_mode() {
    App::new()
        .with_entity(Component1(10))
        .assert_any::<With<Component1>>(1, |e| e.has(|_: &Component2| ()));
}

#[modor_test]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_missing_component() {
    App::new()
        .with_entity(Component1(10))
        .with_entity(Component1(20))
        .with_entity(Singleton1(30))
        .assert::<With<Component1>>(2, |e| e.has_not::<Singleton1>())
        .assert::<With<Singleton1>>(1, |e| e.has_not::<Component1>().has_not::<Component2>())
        .assert_any::<()>(3, |e| e.has_not::<Component1>().has_not::<Singleton1>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> \
have not component integration::app::Component1"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_existing_component() {
    App::new()
        .with_entity(Component1(10))
        .assert::<With<Component1>>(1, |e| e.has_not::<Component1>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> \
have not component integration::app::Component1"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_existing_component_in_any_mode() {
    App::new()
        .with_entity(Component1(10))
        .assert_any::<With<Component1>>(1, |e| e.has_not::<Component1>());
}

#[modor_test]
fn assert_valid_child_count() {
    let entity1 = EntityBuilder::new()
        .component(Component1(10))
        .child_component(Singleton1(20));
    let entity2 = EntityBuilder::new()
        .component(Component2(30))
        .child_component(Singleton2(40))
        .child_entity(EntityBuilder::new());
    App::new()
        .with_entity(entity1)
        .with_entity(entity2)
        .with_entity(Component3(50))
        .assert::<With<Component1>>(1, |e| e.child_count(1))
        .assert::<With<Component2>>(1, |e| e.child_count(2))
        .assert::<With<Singleton1>>(1, |e| e.child_count(0))
        .assert::<With<Singleton2>>(1, |e| e.child_count(0))
        .assert_any::<()>(6, |e| e.child_count(2).child_count(1));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> have 3 children"]
fn assert_invalid_child_count() {
    App::new()
        .with_entity(EntityBuilder::new().child_component(Component1(10)))
        .assert::<With<Component1>>(1, |e| e.child_count(3));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> have 3 children"]
fn assert_invalid_child_count_in_any_mode() {
    App::new()
        .with_entity(EntityBuilder::new().child_component(Component1(10)))
        .assert_any::<With<Component1>>(1, |e| e.child_count(3));
}

#[modor_test]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_matching_parent() {
    let entity = EntityBuilder::new()
        .component(Component1(10))
        .child_component(Singleton1(20));
    App::new()
        .with_entity(entity)
        .assert::<With<Singleton1>>(1, |e| e.has_parent::<With<Component1>>())
        .assert_any::<()>(2, |e| e.has_parent::<With<Component1>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Singleton1> have parent matching \
modor::filters::with::With<integration::app::Component2>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_matching_parent() {
    let entity = EntityBuilder::new()
        .component(Component1(10))
        .child_component(Singleton1(20));
    App::new()
        .with_entity(entity)
        .assert::<With<Singleton1>>(1, |e| e.has_parent::<With<Component2>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Singleton1> have parent matching \
modor::filters::with::With<integration::app::Component2>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_matching_parent_in_any_mode() {
    let entity = EntityBuilder::new()
        .component(Component1(10))
        .child_component(Singleton1(20));
    App::new()
        .with_entity(entity)
        .assert_any::<With<Singleton1>>(1, |e| e.has_parent::<With<Component2>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> have parent matching \
modor::filters::with::With<integration::app::Component1>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_missing_parent() {
    App::new()
        .with_entity(Component1(10))
        .assert::<With<Component1>>(1, |e| e.has_parent::<With<Component1>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Component1> have parent matching \
modor::filters::with::With<integration::app::Component1>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_missing_parent_in_any_mode() {
    App::new()
        .with_entity(Component1(10))
        .assert_any::<With<Component1>>(1, |e| e.has_parent::<With<Component1>>());
}

#[modor_test]
fn update_component() {
    let entity = EntityBuilder::new()
        .component(Component1(10))
        .component(Component2(20));
    App::new()
        .with_entity(entity)
        .with_entity(Component1(30))
        .with_update::<With<Component2>, _>(|c: &mut Component1| c.0 += 5)
        .assert::<With<Component2>>(1, |e| e.has(|c: &Component1| assert_eq!(c.0, 15)))
        .assert::<Not<With<Component2>>>(1, |e| e.has(|c: &Component1| assert_eq!(c.0, 30)));
}

#[modor_test]
fn update_app() {
    let incremental_entity = EntityBuilder::new()
        .component(Component1(10))
        .component(Component1Increment);
    let mut app = App::new()
        .with_entity(incremental_entity)
        .updated()
        .assert::<With<Component1>>(1, |e| e.has(|c: &Component1| assert_eq!(c.0, 11)));
    app.update();
    app.assert::<With<Component1>>(1, |e| e.has(|c: &Component1| assert_eq!(c.0, 12)));
}

#[modor_test(disabled(wasm))]
fn update_app_until_any() {
    let incremental_entity1 = EntityBuilder::new()
        .component(Component1(10))
        .component(Component1Increment);
    let incremental_entity2 = EntityBuilder::new()
        .component(Component1(20))
        .component(Component1Increment);
    App::new()
        .with_entity(incremental_entity1)
        .with_entity(incremental_entity2)
        .updated_until_any::<(), _>(Some(3), |c: &Component1| c.0 == 14)
        .assert_any::<With<Component1>>(2, |e| {
            e.has(|c: &Component1| assert_eq!(c.0, 14))
                .has(|c: &Component1| assert_eq!(c.0, 24))
        })
        .updated_until_any::<(), _>(None, |c: &Component1| c.0 == 30)
        .assert_any::<With<Component1>>(2, |e| {
            e.has(|c: &Component1| assert_eq!(c.0, 20))
                .has(|c: &Component1| assert_eq!(c.0, 30))
        });
}

#[modor_test(disabled(wasm))]
#[should_panic = "max number of retries reached"]
fn update_app_until_any_with_max_retries_reached() {
    let incremental_entity = EntityBuilder::new()
        .component(Component1(10))
        .component(Component1Increment);
    App::new()
        .with_entity(incremental_entity)
        .updated_until_any::<(), _>(Some(3), |c: &Component1| c.0 == 15);
}

#[modor_test]
fn update_app_until_all() {
    let incremental_entity1 = EntityBuilder::new()
        .component(Component1(10))
        .component(Component1Increment);
    let incremental_entity2 = EntityBuilder::new()
        .component(Component1(10))
        .component(Component1Increment);
    App::new()
        .with_entity(incremental_entity1)
        .with_entity(incremental_entity2)
        .updated_until_all::<(), _>(Some(3), |c: &Component1| c.0 == 14)
        .assert::<With<Component1>>(2, |e| e.has(|c: &Component1| assert_eq!(c.0, 14)))
        .updated_until_all::<(), _>(None, |c: &Component1| c.0 == 30)
        .assert::<With<Component1>>(2, |e| e.has(|c: &Component1| assert_eq!(c.0, 30)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "max number of retries reached"]
fn update_app_until_all_with_max_retries_reached() {
    let incremental_entity1 = EntityBuilder::new()
        .component(Component1(10))
        .component(Component1Increment);
    let incremental_entity2 = EntityBuilder::new()
        .component(Component1(20))
        .component(Component1Increment);
    App::new()
        .with_entity(incremental_entity1)
        .with_entity(incremental_entity2)
        .updated_until_all::<(), _>(Some(3), |c: &Component1| c.0 == 14);
}

#[modor_test(disabled(wasm))]
fn update_components() {
    let mut app = App::new()
        .with_entity(Component1(10))
        .with_entity(Component1(20));
    app.update_components(|c: &mut Component1| c.0 += 1);
    app.assert_any::<With<Component1>>(2, |e| {
        e.has(|c: &Component1| assert_eq!(c.0, 11))
            .has(|c: &Component1| assert_eq!(c.0, 21))
    });
}

#[modor_test]
fn start_runner() {
    let mut run = false;
    App::new().run(|_| run = true);
    assert!(run);
}

#[modor_test(disabled(wasm))]
fn add_component() {
    App::new()
        .with_entity(Singleton1(10))
        .with_entity(Singleton2(20))
        .with_component::<With<Singleton1>, _>(|| Component1(30))
        .assert::<With<Singleton1>>(1, |e| e.has::<Component1>(|c| assert_eq!(c.0, 30)))
        .with_component::<With<Singleton1>, _>(|| Component1(40))
        .assert::<With<Singleton1>>(1, |e| e.has::<Component1>(|c| assert_eq!(c.0, 40)));
}

#[modor_test(disabled(wasm))]
fn delete_component() {
    let entity = EntityBuilder::new()
        .component(Component1(10))
        .component(Component2(20));
    App::new()
        .with_entity(entity)
        .with_entity(Component2(30))
        .with_deleted_components::<With<Component1>, Component2>()
        .assert::<With<Component1>>(1, |e| e)
        .assert::<With<Component2>>(1, |e| e);
}

#[modor_test(disabled(wasm))]
fn delete_entity() {
    App::new()
        .with_entity(Component1(10))
        .with_entity(Component2(20))
        .with_deleted_entities::<With<Component1>>()
        .assert::<With<Component1>>(0, |e| e)
        .assert::<With<Component2>>(1, |e| e);
}

#[derive(Component, NoSystem)]
struct Component1(usize);

#[derive(Component, NoSystem)]
struct Component2(usize);

#[derive(Component, NoSystem)]
struct Component3(usize);

#[derive(SingletonComponent, NoSystem)]
struct Singleton1(usize);

#[derive(SingletonComponent, NoSystem)]
struct Singleton2(usize);

#[derive(Component)]
struct Component1Increment;

#[systems]
impl Component1Increment {
    #[run]
    fn update(component: &mut Component1) {
        component.0 += 1;
    }
}
