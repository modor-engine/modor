use log::LevelFilter;
use modor::{App, BuiltEntity, EntityBuilder, With};

#[derive(SingletonComponent, NoSystem)]
struct Singleton(u32);

#[derive(Component, NoSystem)]
struct Component(u32);

#[derive(Component, NoSystem)]
struct OtherComponent(u32);

#[derive(Component, NoSystem)]
struct UnusedComponent;

#[derive(Component, NoSystem)]
struct Entity1;

#[derive(Component, NoSystem)]
struct Entity2;

#[derive(Component)]
struct Entity;

#[systems]
impl Entity {
    fn build(value: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Component(value))
    }

    fn build_entity1(value: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Component(value))
            .with(Entity1)
    }

    fn build_entity2(value: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Component(value))
            .with(Entity2)
    }

    fn build_with_children(value: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Component(value))
            .with_child(EntityBuilder::new().with(Child1))
            .with_child(EntityBuilder::new().with(Child2))
    }

    #[run]
    fn update(component: &mut Component) {
        component.0 += 1;
    }
}

#[derive(Component, NoSystem)]
struct Child1;

#[derive(Component, NoSystem)]
struct Child2;

#[derive(Component, NoSystem)]
struct Child(u32);

#[derive(Component)]
struct Counter(u32);

#[systems]
impl Counter {
    #[run]
    fn update(&mut self) {
        self.0 += 1;
    }
}

#[modor_test(disabled(wasm))]
fn create_app_with_thread_count_and_log_level() {
    let app = App::new()
        .with_thread_count(2)
        .with_log_level(LevelFilter::Info);
    assert_eq!(app.thread_count(), 2);
    let app = app.with_thread_count(1);
    assert_eq!(app.thread_count(), 1);
    let app = app.with_thread_count(0);
    assert_eq!(app.thread_count(), 1);
}

#[modor_test(disabled(windows, linux, macos, android))]
fn create_app_with_thread_count_and_log_level_for_wasm() {
    let app = App::new()
        .with_thread_count(2)
        .with_log_level(LevelFilter::Info);
    assert_eq!(app.thread_count(), 1);
}

#[modor_test]
fn assert_valid_entity_count() {
    App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build(20))
        .with_entity(Singleton(30))
        .assert::<With<Entity>>(2, |e| e)
        .assert::<With<Singleton>>(1, |e| e)
        .assert::<(With<Entity>, With<Component>)>(2, |e| e)
        .assert::<(With<Singleton>, With<Entity>)>(0, |e| e)
        .assert::<With<UnusedComponent>>(0, |e| e);
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: 2 entities matching \
modor::filters::with::With<integration::app::Entity>, actual count: 1"]
fn assert_invalid_entity_count() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(2, |e| e);
}

#[modor_test(disabled(wasm))]
fn assert_entity_has_existing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build(20))
        .with_entity(Singleton(30))
        .assert::<With<Entity>>(2, |e| {
            e.has(|c: &Component| assert!(c.0 == 10 || c.0 == 20))
        })
        .assert_any::<With<Entity>>(2, |e| {
            e.has(|c: &Component| assert_eq!(c.0, 10))
                .has(|c: &Component| assert_eq!(c.0, 20))
                .has(|c: &Component| assert!(c.0 > 0))
        })
        .assert::<With<Singleton>>(1, |e| e.has(|c: &Singleton| assert_eq!(c.0, 30)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: `(left == right)"]
fn assert_entity_has_invalid_component() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 20)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: `(left == right)"]
fn assert_entity_has_invalid_component_in_any_mode() {
    App::new()
        .with_entity(Entity::build(10))
        .assert_any::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 20)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have component integration::app::Singleton"]
fn assert_entity_has_missing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.has(|_: &Singleton| ()));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have component integration::app::Singleton"]
fn assert_entity_has_missing_component_in_any_mode() {
    App::new()
        .with_entity(Entity::build(10))
        .assert_any::<With<Entity>>(1, |e| e.has(|_: &Singleton| ()));
}

#[modor_test]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_missing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build(20))
        .with_entity(Singleton(30))
        .assert::<With<Entity>>(2, |e| e.has_not::<Singleton>())
        .assert::<With<Singleton>>(1, |e| e.has_not::<Component>().has_not::<UnusedComponent>())
        .assert_any::<()>(3, |e| e.has_not::<Entity>().has_not::<Singleton>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have not component integration::app::Component"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_existing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.has_not::<Component>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have not component integration::app::Component"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_existing_component_in_any_mode() {
    App::new()
        .with_entity(Entity::build(10))
        .assert_any::<With<Entity>>(1, |e| e.has_not::<Component>());
}

#[modor_test]
fn assert_valid_child_count() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .with_entity(Entity::build_with_children(20))
        .with_entity(Singleton(30))
        .assert::<With<Entity>>(2, |e| e.child_count(2))
        .assert_any::<()>(7, |e| e.child_count(2));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have 3 children"]
fn assert_invalid_child_count() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Entity>>(1, |e| e.child_count(3));
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have 3 children"]
fn assert_invalid_child_count_in_any_mode() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert_any::<With<Entity>>(1, |e| e.child_count(3));
}

#[modor_test]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_matching_parent() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Child1>>(1, |e| e.has_parent::<With<Component>>())
        .assert::<With<Child2>>(1, |e| e.has_parent::<With<Entity>>())
        .assert_any::<()>(3, |e| e.has_parent::<With<Entity>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Child1> have parent matching \
modor::filters::with::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_matching_parent() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Child1>>(1, |e| e.has_parent::<With<Singleton>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Child1> have parent matching \
modor::filters::with::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_matching_parent_in_any_mode() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert_any::<With<Child1>>(1, |e| e.has_parent::<With<Singleton>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have parent matching \
modor::filters::with::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_missing_parent() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Entity>>(1, |e| e.has_parent::<With<Singleton>>());
}

#[modor_test(disabled(wasm))]
#[should_panic = "assertion failed: entities matching \
modor::filters::with::With<integration::app::Entity> have parent matching \
modor::filters::with::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_missing_parent_in_any_mode() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert_any::<With<Entity>>(1, |e| e.has_parent::<With<Singleton>>());
}

#[modor_test]
fn update_component() {
    App::new()
        .with_entity(Entity::build_entity1(10))
        .with_entity(Entity::build_entity2(20))
        .with_update::<With<Entity1>, _>(|c: &mut Component| c.0 += 5)
        .assert::<With<Entity1>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 15)))
        .assert::<With<Entity2>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 20)));
}

#[modor_test]
fn update_app() {
    let mut app = App::new()
        .with_entity(Entity::build(10))
        .updated()
        .assert::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 11)));
    app.update();
    app.assert::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 12)));
}

#[modor_test(disabled(wasm))]
fn update_app_until_any() {
    App::new()
        .with_entity(Counter(0))
        .with_entity(Counter(1))
        .updated_until_any::<(), _>(Some(3), |c: &Counter| c.0 == 5)
        .assert_any::<With<Counter>>(2, |e| {
            e.has(|c: &Counter| assert_eq!(c.0, 4))
                .has(|c: &Counter| assert_eq!(c.0, 5))
        })
        .updated_until_any::<(), _>(None, |c: &Counter| c.0 == 15)
        .assert_any::<With<Counter>>(2, |e| {
            e.has(|c: &Counter| assert_eq!(c.0, 14))
                .has(|c: &Counter| assert_eq!(c.0, 15))
        });
}

#[modor_test(disabled(wasm))]
#[should_panic = "max number of retries reached"]
fn update_app_until_any_with_max_retries_reached() {
    App::new()
        .with_entity(Counter(0))
        .with_entity(Counter(1))
        .updated_until_any::<(), _>(Some(2), |c: &Counter| c.0 == 5);
}

#[modor_test]
fn update_app_until_all() {
    App::new()
        .with_entity(Counter(1))
        .with_entity(Counter(1))
        .updated_until_all::<(), _>(Some(3), |c: &Counter| c.0 == 5)
        .assert::<With<Counter>>(2, |e| e.has(|c: &Counter| assert_eq!(c.0, 5)))
        .updated_until_all::<(), _>(None, |c: &Counter| c.0 == 15)
        .assert::<With<Counter>>(2, |e| e.has(|c: &Counter| assert_eq!(c.0, 15)));
}

#[modor_test(disabled(wasm))]
#[should_panic = "max number of retries reached"]
fn update_app_until_all_with_max_retries_reached() {
    App::new()
        .with_entity(Counter(1))
        .with_entity(Counter(2))
        .updated_until_all::<(), _>(Some(2), |c: &Counter| c.0 == 5);
}

#[modor_test(disabled(wasm))]
fn update_components() {
    let mut app = App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build_entity1(20))
        .with_entity(Entity::build_entity2(30));
    app.update_components(|c: &mut Component| c.0 += 1);
    app.assert_any::<With<Component>>(3, |e| {
        e.has(|c: &Component| assert_eq!(c.0, 11))
            .has(|c: &Component| assert_eq!(c.0, 21))
            .has(|c: &Component| assert_eq!(c.0, 31))
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
        .with_entity(Entity::build_entity1(10))
        .with_entity(Entity::build_entity2(20))
        .with_component::<With<Entity1>, _>(|| OtherComponent(0))
        .assert::<With<OtherComponent>>(1, |e| {
            e.has::<Entity1, _>(|_| ())
                .has(|c: &OtherComponent| assert_eq!(c.0, 0))
        })
        .with_component::<With<Entity1>, _>(|| OtherComponent(1))
        .assert::<With<OtherComponent>>(1, |e| {
            e.has::<Entity1, _>(|_| ())
                .has(|c: &OtherComponent| assert_eq!(c.0, 1))
        });
}

#[modor_test(disabled(wasm))]
fn delete_component() {
    App::new()
        .with_entity(Entity::build_entity1(10))
        .with_entity(Entity::build_entity2(20))
        .with_deleted_components::<With<Entity1>, Entity1>()
        .assert::<With<Entity1>>(0, |e| e)
        .assert::<With<Entity2>>(1, |e| e)
        .assert::<With<Entity>>(2, |e| e);
}

#[modor_test(disabled(wasm))]
fn delete_entity() {
    App::new()
        .with_entity(Entity::build_entity1(10))
        .with_entity(Entity::build_entity2(20))
        .with_deleted_entities::<With<Entity1>>()
        .assert::<With<Entity1>>(0, |e| e)
        .assert::<With<Entity2>>(1, |e| e)
        .assert::<With<Entity>>(1, |e| e);
}
