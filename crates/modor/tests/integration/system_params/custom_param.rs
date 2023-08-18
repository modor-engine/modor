use crate::system_params::{OtherValue, Value};
use modor::{App, BuiltEntity, Custom, EntityBuilder, Query, With};

#[test]
fn use_named_custom_param() {
    let tester = EntityBuilder::new()
        .component(NamedTester::default())
        .component(Value(1));
    App::new()
        .with_entity(tester)
        .with_entity(OtherValue(5))
        .with_entity(OtherValue(2))
        .with_entity(OtherValue(3))
        .updated()
        .assert::<With<NamedTester>>(1, |e| {
            e.has(|t: &NamedTester| assert_eq!(t.other_value_sum, 10))
                .has(|v: &Value| assert_eq!(v.0, 2))
        });
}

#[test]
fn use_unnamed_custom_param() {
    let tester = EntityBuilder::new()
        .component(UnnamedTester::default())
        .component(Value(1));
    App::new()
        .with_entity(tester)
        .with_entity(OtherValue(5))
        .with_entity(OtherValue(2))
        .with_entity(OtherValue(3))
        .updated()
        .assert::<With<UnnamedTester>>(1, |e| {
            e.has(|t: &UnnamedTester| assert_eq!(t.other_value_sum, 10))
                .has(|v: &Value| assert_eq!(v.0, 2))
        });
}

#[test]
fn use_unit_custom_param() {
    App::new()
        .with_entity(UnitTester::default())
        .updated()
        .assert::<With<UnitTester>>(1, |e| e.has(|t: &UnitTester| assert!(t.is_run)));
}

#[derive(SystemParam)]
struct NamedSystemParam<'a> {
    value: &'a mut Value,
    query: Query<'a, &'static OtherValue>,
}

#[derive(Component, Default)]
struct NamedTester {
    other_value_sum: u32,
}

#[systems]
impl NamedTester {
    #[run]
    fn update(&mut self, mut param: Custom<NamedSystemParam<'_>>) {
        param.value.0 += 1;
        self.other_value_sum = param.query.iter().map(|o| o.0).sum();
    }
}

#[derive(SystemParam)]
struct UnnamedSystemParam<'a>(&'a mut Value, Query<'a, &'static OtherValue>);

#[derive(Component, Default)]
struct UnnamedTester {
    other_value_sum: u32,
}

#[systems]
impl UnnamedTester {
    #[run]
    fn update(&mut self, mut param: Custom<UnnamedSystemParam<'_>>) {
        param.0 .0 += 1;
        self.other_value_sum = param.1.iter().map(|o| o.0).sum();
    }
}

#[derive(SystemParam)]
struct UnitSystemParam;

#[derive(Component, Default)]
struct UnitTester {
    is_run: bool,
}

#[systems]
impl UnitTester {
    #[run]
    fn update(&mut self, _param: Custom<UnitSystemParam>) {
        self.is_run = true;
    }
}
