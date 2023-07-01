use crate::system_params::{BuiltEntity, Number, OtherNumber, Text, Value};
use modor::{App, Filter, SingleMut, With};

#[derive(SingletonComponent, NoSystem, Default)]
struct Numbers(Vec<u32>);

#[derive(Component)]
struct RegisteredNumber;

#[systems]
impl RegisteredNumber {
    #[run]
    fn collect(value: &Value, mut collector: SingleMut<'_, Numbers>, _filter: Filter<With<Text>>) {
        collector.0.push(value.0);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

#[modor_test]
fn iterate_using_filter() {
    App::new()
        .with_entity(Numbers::default())
        .with_entity(Number::build(1).component(RegisteredNumber))
        .with_entity(OtherNumber::build(10))
        .with_entity(Number::build(2).component(RegisteredNumber))
        .with_entity(Number::build_without_value().component(RegisteredNumber))
        .with_entity(Number::build_with_additional_component(3).component(RegisteredNumber))
        .updated()
        .assert::<With<Numbers>>(1, |e| e.has(|c: &Numbers| assert_eq!(c.0, [3])));
}
