use log::LevelFilter;
use modor::{App, Built, EntityBuilder, With};

struct Singleton(u32);

#[singleton]
impl Singleton {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value))
    }
}

struct Component(u32);

struct Entity1;

struct Entity2;

struct Entity;

#[entity]
impl Entity {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Component(value))
    }

    fn build_entity1(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Component(value))
            .with(Entity1)
    }

    fn build_entity2(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Component(value))
            .with(Entity2)
    }

    fn build_with_children(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Component(value))
            .with_child(Child::build1(value + 1))
            .with_child(Child::build2(value + 2))
    }

    #[run]
    fn update(component: &mut Component) {
        component.0 += 1;
    }
}

struct Child1;

struct Child2;

struct Child(u32);

#[entity]
impl Child {
    fn build1(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value)).with(Child1)
    }

    fn build2(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value)).with(Child2)
    }
}

struct Counter(u32);

#[entity]
impl Counter {
    fn build(initial_value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(initial_value))
    }

    #[run]
    fn update(&mut self) {
        self.0 += 1;
    }
}

#[test]
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

#[allow(dead_code)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_app_with_thread_count_and_log_level_for_wasm() {
    let app = App::new()
        .with_thread_count(2)
        .with_log_level(LevelFilter::Info);
    assert_eq!(app.thread_count(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn assert_valid_entity_count() {
    App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build(20))
        .with_entity(Singleton::build(30))
        .assert::<With<Entity>>(2, |e| e)
        .assert::<With<Singleton>>(1, |e| e)
        .assert::<(With<Entity>, With<Component>)>(2, |e| e)
        .assert::<(With<Singleton>, With<Entity>)>(0, |e| e)
        .assert::<With<usize>>(0, |e| e);
}

#[test]
#[should_panic = "assertion failed: 2 entities matching \
modor::entities::filters::With<integration::app::Entity>, actual count: 1"]
fn assert_invalid_entity_count() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(2, |e| e);
}

#[test]
fn assert_entity_has_existing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build(20))
        .with_entity(Singleton::build(30))
        .assert::<With<Entity>>(2, |e| {
            e.has(|c: &Component| assert!(c.0 == 10 || c.0 == 20))
                .any()
                .has(|c: &Component| assert_eq!(c.0, 10))
                .has(|c: &Component| assert_eq!(c.0, 20))
        })
        .assert::<With<Singleton>>(1, |e| e.has(|c: &Singleton| assert_eq!(c.0, 30)));
}

#[test]
#[should_panic = "assertion failed: `(left == right)"]
fn assert_entity_has_invalid_component() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 20)));
}

#[test]
#[should_panic = "assertion failed: `(left == right)"]
fn assert_entity_has_invalid_component_in_any_mode() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.any().has(|c: &Component| assert_eq!(c.0, 20)));
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have component integration::app::Singleton"]
fn assert_entity_has_missing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.has(|_: &Singleton| ()));
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have component integration::app::Singleton"]
fn assert_entity_has_missing_component_in_any_mode() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.any().has(|_: &Singleton| ()));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_missing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .with_entity(Entity::build(20))
        .with_entity(Singleton::build(30))
        .assert::<With<Entity>>(2, |e| e.has_not::<Singleton>())
        .assert::<With<Singleton>>(1, |e| e.has_not::<Component>().has_not::<usize>())
        .assert::<()>(3, |e| e.any().has_not::<Entity>().has_not::<Singleton>());
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have not component integration::app::Component"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_existing_component() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.has_not::<Component>());
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have not component integration::app::Component"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_existing_component_in_any_mode() {
    App::new()
        .with_entity(Entity::build(10))
        .assert::<With<Entity>>(1, |e| e.any().has_not::<Component>());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn assert_valid_child_count() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .with_entity(Entity::build_with_children(20))
        .with_entity(Singleton::build(30))
        .assert::<With<Entity>>(2, |e| e.child_count(2))
        .assert::<()>(7, |e| e.any().child_count(2));
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have 3 children"]
fn assert_invalid_child_count() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Entity>>(1, |e| e.child_count(3));
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have 3 children"]
fn assert_invalid_child_count_in_any_mode() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Entity>>(1, |e| e.any().child_count(3));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_matching_parent() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Child1>>(1, |e| e.has_parent::<With<Component>>())
        .assert::<With<Child2>>(1, |e| e.has_parent::<With<Entity>>())
        .assert::<()>(3, |e| e.any().has_parent::<With<Entity>>());
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Child1> have parent matching \
modor::entities::filters::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_matching_parent() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Child1>>(1, |e| e.has_parent::<With<Singleton>>());
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Child1> have parent matching \
modor::entities::filters::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_not_matching_parent_in_any_mode() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Child1>>(1, |e| e.any().has_parent::<With<Singleton>>());
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have parent matching \
modor::entities::filters::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_missing_parent() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Entity>>(1, |e| e.has_parent::<With<Singleton>>());
}

#[test]
#[should_panic = "assertion failed: entities matching \
modor::entities::filters::With<integration::app::Entity> have parent matching \
modor::entities::filters::With<integration::app::Singleton>"]
#[allow(clippy::redundant_closure_for_method_calls)]
fn assert_entity_has_missing_parent_in_any_mode() {
    App::new()
        .with_entity(Entity::build_with_children(10))
        .assert::<With<Entity>>(1, |e| e.any().has_parent::<With<Singleton>>());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_component() {
    App::new()
        .with_entity(Entity::build_entity1(10))
        .with_entity(Entity::build_entity2(20))
        .with_update::<With<Entity1>, _>(|c: &mut Component| c.0 += 5)
        .assert::<With<Entity1>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 15)))
        .assert::<With<Entity2>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 20)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_app() {
    let mut app = App::new()
        .with_entity(Entity::build(10))
        .updated()
        .assert::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 11)));
    app.update();
    app.assert::<With<Entity>>(1, |e| e.has(|c: &Component| assert_eq!(c.0, 12)));
}

#[test]
fn update_app_until_any() {
    App::new()
        .with_entity(Counter::build(0))
        .with_entity(Counter::build(1))
        .updated_until_any::<(), _>(Some(3), |c: &Counter| c.0 == 5)
        .assert::<With<Counter>>(2, |e| {
            e.any()
                .has(|c: &Counter| assert_eq!(c.0, 4))
                .has(|c: &Counter| assert_eq!(c.0, 5))
        })
        .updated_until_any::<(), _>(None, |c: &Counter| c.0 == 15)
        .assert::<With<Counter>>(2, |e| {
            e.any()
                .has(|c: &Counter| assert_eq!(c.0, 14))
                .has(|c: &Counter| assert_eq!(c.0, 15))
        });
}

#[test]
#[should_panic = "max number of retries reached"]
fn update_app_until_any_with_max_retries_reached() {
    let _app = App::new()
        .with_entity(Counter::build(0))
        .with_entity(Counter::build(1))
        .updated_until_any::<(), _>(Some(2), |c: &Counter| c.0 == 5);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_app_until_all() {
    App::new()
        .with_entity(Counter::build(1))
        .with_entity(Counter::build(1))
        .updated_until_all::<(), _>(Some(3), |c: &Counter| c.0 == 5)
        .assert::<With<Counter>>(2, |e| e.has(|c: &Counter| assert_eq!(c.0, 5)))
        .updated_until_all::<(), _>(None, |c: &Counter| c.0 == 15)
        .assert::<With<Counter>>(2, |e| e.has(|c: &Counter| assert_eq!(c.0, 15)));
}

#[test]
#[should_panic = "max number of retries reached"]
fn update_app_until_all_with_max_retries_reached() {
    let _app = App::new()
        .with_entity(Counter::build(1))
        .with_entity(Counter::build(2))
        .updated_until_all::<(), _>(Some(2), |c: &Counter| c.0 == 5);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_singleton() {
    let mut app = App::new().with_entity(Singleton::build(10));
    app.update_singleton(|s: &mut Singleton| s.0 = 20);
    app.assert::<With<Singleton>>(1, |e| e.has(|c: &Singleton| assert_eq!(c.0, 20)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn start_runner() {
    let mut run = false;
    App::new().run(|_| run = true);
    assert!(run);
}
