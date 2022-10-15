use modor::{App, Built, EntityBuilder, With};

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
            .with_option(add_option.then_some(42_u32))
            .with_option((!add_option).then_some(42_i8))
            .with_dependency(Singleton1::build(10))
            .with_dependency(Singleton2::build(20))
            .with_dependency(Singleton3::build(30))
    }
}

struct InheritedChild;

struct Inherited;

#[entity]
impl Inherited {
    fn build(value: i64) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(value)
            .with_child(Self::build_child(value + 1))
    }

    fn build_child(value: i64) -> impl Built<Self> {
        EntityBuilder::new(Self).with(value).with(InheritedChild)
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_complex_entities() {
    App::new()
        .with_entity(Singleton1::build(40))
        .with_entity(Singleton1::build(41))
        .with_entity(Level1::build(100))
        .with_entity(Singleton3::build(50))
        .assert::<With<Singleton1>>(1, |e| e.has(|c: &Singleton1| assert_eq!(c.0, 41)))
        .assert::<With<Singleton2>>(1, |e| e.has(|c: &Singleton2| assert_eq!(c.0, 20)))
        .assert::<With<Singleton3>>(1, |e| e.has(|c: &Singleton3| assert_eq!(c.0, 50)))
        .assert::<With<Level1>>(1, |e| {
            e.has(|c: &Value| assert_eq!(c.0, 100)).child_count(1)
        })
        .assert::<With<Level2>>(1, |e| {
            e.has(|_: &Inherited| ())
                .has(|c: &Value| assert_eq!(c.0, 201))
                .has(|c: &i64| assert_eq!(c, &101))
                .has_parent::<With<Level1>>()
                .child_count(3)
        })
        .assert::<With<InheritedChild>>(1, |e| {
            e.has(|_: &Inherited| ())
                .has(|c: &i64| assert_eq!(c, &102))
                .has_parent::<With<Level2>>()
                .child_count(0)
        })
        .assert::<(With<Level3>, With<u32>)>(1, |e| {
            e.has(|v: &Value| assert_eq!(v.0, 103))
                .has(|v: &u32| assert_eq!(v, &42))
                .has_not::<i8>()
                .has_parent::<With<Level2>>()
                .child_count(0)
        })
        .assert::<(With<Level3>, With<i8>)>(1, |e| {
            e.has(|v: &Value| assert_eq!(v.0, 104))
                .has(|v: &i8| assert_eq!(v, &42))
                .has_not::<u32>()
                .has_parent::<With<Level2>>()
                .child_count(0)
        });
}
