use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};

struct Singleton1(u32);

#[singleton]
impl Singleton1 {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value))
    }
}

struct Singleton2(u32);

#[singleton]
impl Singleton2 {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value))
    }
}

struct Singleton3(u32);

#[singleton]
impl Singleton3 {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value))
    }
}

struct Value(u32);

struct Level1;

#[entity]
impl Level1 {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value(value))
            .with_child(Level2::build(value + 1))
    }
}

struct Level2;

#[entity]
impl Level2 {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Inherited::build(i64::from(value)))
            .with(Value(value + 100))
            .with_children(move |a| {
                for i in 2..4 {
                    a.add(Level3::build(value + i, i == 2));
                }
            })
    }
}

struct Level3;

#[entity]
impl Level3 {
    fn build(value: u32, add_option: bool) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value(value))
            .with_option(add_option.then(|| 42_u32))
            .with_dependency(Singleton1::build(10))
            .with_dependency(Singleton2::build(20))
            .with_dependency(Singleton3::build(30))
    }
}

struct Inherited;

#[entity]
impl Inherited {
    fn build(value: i64) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(value)
            .with_child(Self::build_child(value + 1))
    }

    fn build_child(value: i64) -> impl Built<Self> {
        EntityBuilder::new(Self).with(value)
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_complex_entities() {
    let mut app: TestApp = App::new()
        .with_entity(Singleton1::build(40))
        .with_entity(Singleton1::build(41))
        .into();
    let entity_id = app.create_entity(Level1::build(100));
    app.create_entity(Singleton3::build(50));
    app.assert_singleton::<Singleton1>()
        .has(|s: &Singleton1| assert_eq!(s.0, 41));
    app.assert_singleton::<Singleton2>()
        .has(|s: &Singleton2| assert_eq!(s.0, 20));
    app.assert_singleton::<Singleton3>()
        .has(|s: &Singleton3| assert_eq!(s.0, 50));
    app.assert_entity(entity_id)
        .has(|_: &Level1| ())
        .has(|v: &Value| assert_eq!(v.0, 100))
        .has_children(|c| assert_eq!(c, [entity_id + 1]));
    app.assert_entity(entity_id + 1)
        .has(|_: &Level2| ())
        .has(|_: &Inherited| ())
        .has(|v: &Value| assert_eq!(v.0, 201))
        .has(|v: &i64| assert_eq!(v, &101))
        .has_children(|c| assert_eq!(c, [entity_id + 2, entity_id + 3, entity_id + 6]));
    app.assert_entity(entity_id + 2)
        .has(|_: &Inherited| ())
        .has(|v: &i64| assert_eq!(v, &102));
    app.assert_entity(entity_id + 3)
        .has(|_: &Level3| ())
        .has(|v: &Value| assert_eq!(v.0, 103))
        .has(|v: &u32| assert_eq!(v, &42))
        .has_children(|c| assert_eq!(c, []));
    app.assert_entity(entity_id + 6)
        .has(|_: &Level3| ())
        .has(|v: &Value| assert_eq!(v.0, 104))
        .has_not::<u32>()
        .has_children(|c| assert_eq!(c, []));
}
