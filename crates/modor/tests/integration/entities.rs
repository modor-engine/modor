use modor::{App, BuiltEntity, EntityBuilder, LevelFilter, With};

#[derive(Component, NoSystem)]
struct I64(i64);

#[derive(Component, NoSystem)]
struct U32(u32);

#[derive(Component, NoSystem)]
struct I8(i8);

#[derive(SingletonComponent, NoSystem)]
struct Singleton1(u32);

#[derive(SingletonComponent, NoSystem)]
struct Singleton2(u32);

#[derive(SingletonComponent, NoSystem)]
struct Singleton3(u32);

#[derive(Component, NoSystem)]
struct Value(u32);

#[derive(Component, NoSystem)]
struct Level1;

impl Level1 {
    fn build(value: u32) -> impl BuiltEntity {
        EntityBuilder::default()
            .with(Self)
            .with(Value(value))
            .with_child(Level2::build(value + 1))
    }
}

#[derive(Component, NoSystem)]
struct Level2;

impl Level2 {
    fn build(value: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with_inherited(Inherited::build(i64::from(value)))
            .with(Value(value + 100))
            .with_children(move |b| {
                for i in 2..4 {
                    b.add(Level3::build(value + i, i == 2));
                }
            })
    }
}

#[derive(Component, NoSystem)]
struct Level3;

impl Level3 {
    fn build(value: u32, add_option: bool) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Value(value))
            .with_option(add_option.then_some(42_u32).map(U32))
            .with_option((!add_option).then_some(42_i8).map(I8))
            .with_dependency::<Singleton1, _, _>(|| Singleton1(10))
            .with_dependency::<Singleton2, _, _>(|| Singleton2(20))
            .with_dependency::<Singleton3, _, _>(|| Singleton3(30))
    }
}

#[derive(Component, NoSystem)]
struct InheritedChild;

#[derive(Component, NoSystem)]
struct Inherited;

impl Inherited {
    fn build(value: i64) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(I64(value))
            .with_child(Self::build_child(value + 1))
    }

    fn build_child(value: i64) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(I64(value))
            .with(InheritedChild)
    }
}

#[modor_test]
fn create_complex_entities() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(Singleton1(40))
        .with_entity(Singleton1(41))
        .with_entity(Level1::build(100))
        .with_entity(Singleton3(50))
        .assert::<With<Singleton1>>(1, |e| e.has(|c: &Singleton1| assert_eq!(c.0, 41)))
        .assert::<With<Singleton2>>(1, |e| e.has(|c: &Singleton2| assert_eq!(c.0, 20)))
        .assert::<With<Singleton3>>(1, |e| e.has(|c: &Singleton3| assert_eq!(c.0, 50)))
        .assert::<With<Level1>>(1, |e| {
            e.has(|c: &Value| assert_eq!(c.0, 100)).child_count(1)
        })
        .assert::<With<Level2>>(1, |e| {
            e.has(|_: &Inherited| ())
                .has(|c: &Value| assert_eq!(c.0, 201))
                .has(|c: &I64| assert_eq!(c.0, 101))
                .has_parent::<With<Level1>>()
                .child_count(3)
        })
        .assert::<With<InheritedChild>>(1, |e| {
            e.has(|_: &Inherited| ())
                .has(|c: &I64| assert_eq!(c.0, 102))
                .has_parent::<With<Level2>>()
                .child_count(0)
        })
        .assert::<(With<Level3>, With<U32>)>(1, |e| {
            e.has(|v: &Value| assert_eq!(v.0, 103))
                .has(|v: &U32| assert_eq!(v.0, 42))
                .has_not::<I8>()
                .has_parent::<With<Level2>>()
                .child_count(0)
        })
        .assert::<(With<Level3>, With<I8>)>(1, |e| {
            e.has(|v: &Value| assert_eq!(v.0, 104))
                .has(|v: &I8| assert_eq!(v.0, 42))
                .has_not::<U32>()
                .has_parent::<With<Level2>>()
                .child_count(0)
        });
}

#[modor_test]
fn create_entity_with_same_singleton_in_children() {
    App::new()
        .with_entity(
            EntityBuilder::new()
                .with(Singleton1(0))
                .with_child(Singleton1(1))
                .with_child(Singleton1(2)),
        )
        .assert::<With<Singleton1>>(0, |e| e);
}
