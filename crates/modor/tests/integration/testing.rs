use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};

struct Value(u32);

struct EntityExample;

#[entity]
impl EntityExample {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value(value))
            .with_child(Self::build_child(value))
    }

    fn build_child(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(value)
    }
}

struct SingletonExample;

#[singleton]
impl SingletonExample {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value(value))
            .with_child(EntityExample::build_child(value))
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_entity_and_assert() {
    let mut app = TestApp::new();
    let entity_id = app.create_entity(EntityExample::build_child(10));
    assert_eq!(app.thread_count(), 1);
    assert_eq!(entity_id, 0);
    app.assert_entity(entity_id)
        .exists()
        .has(|v: &u32| assert_eq!(v, &10))
        .has_not::<Value>()
        .has_children(|c| assert_eq!(c, []));
    app.assert_entity(10).does_not_exist();
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_child_and_assert() {
    let app: App = TestApp::new().into();
    let mut app: TestApp = app.with_entity(EntityExample::build(10)).into();
    let entity_id = app.create_child(0, EntityExample::build(20));
    assert_eq!(entity_id, 2);
    app.assert_entity(entity_id)
        .exists()
        .has(|v: &Value| assert_eq!(v.0, 20))
        .has_children(|c| assert_eq!(c, [3]));
}

#[test]
#[should_panic]
fn assert_child_with_missing_parent() {
    let mut app = TestApp::new();
    app.create_child(0, EntityExample::build(10));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn assert_singleton() {
    let app: TestApp = App::new().with_entity(SingletonExample::build(30)).into();
    app.assert_singleton::<SingletonExample>()
        .exists()
        .has(|v: &Value| assert_eq!(v.0, 30))
        .has_not::<u32>()
        .has_children(|c| assert_eq!(c, [1]));
}

#[test]
#[should_panic]
fn assert_exists_for_missing_entity() {
    let app = TestApp::new();
    app.assert_entity(0).exists();
}

#[test]
#[should_panic]
fn assert_existing_entity_does_not_exist() {
    let mut app = TestApp::new();
    let entity_id = app.create_entity(EntityExample::build_child(10));
    app.assert_entity(entity_id).does_not_exist();
}

#[test]
#[should_panic]
fn assert_missing_entity_has_component() {
    let app = TestApp::new();
    app.assert_entity(0).has(|_: &Value| ());
}

#[test]
#[should_panic]
fn assert_has_missing_component() {
    let mut app = TestApp::new();
    let entity_id = app.create_entity(EntityExample::build_child(10));
    app.assert_entity(entity_id).has(|_: &Value| ());
}

#[test]
#[should_panic]
fn assert_has_component_with_wrong_assertion() {
    let mut app = TestApp::new();
    let entity_id = app.create_entity(EntityExample::build_child(10));
    app.assert_entity(entity_id)
        .has(|v: &u32| assert_eq!(v, &20));
}

#[test]
#[should_panic]
fn assert_missing_entity_has_not_component() {
    let app = TestApp::new();
    app.assert_entity(0).has_not::<Value>();
}

#[test]
#[should_panic]
fn assert_has_not_existing_component() {
    let mut app = TestApp::new();
    let entity_id = app.create_entity(EntityExample::build_child(10));
    app.assert_entity(entity_id).has_not::<u32>();
}

#[test]
#[should_panic]
fn assert_missing_entity_has_children() {
    let app = TestApp::new();
    app.assert_entity(0).has_children(|_| ());
}

#[test]
#[should_panic]
fn assert_has_children_with_wrong_assertion() {
    let mut app = TestApp::new();
    let entity_id = app.create_entity(EntityExample::build_child(10));
    app.assert_entity(entity_id)
        .has_children(|c| assert_eq!(c.len(), 1));
}
