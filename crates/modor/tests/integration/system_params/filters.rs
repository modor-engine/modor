use modor::{App, Built, EntityBuilder, Filter, SingleMut, With};

struct StreamCollector(Vec<u32>);

#[singleton]
impl StreamCollector {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(vec![]))
    }
}

struct Value(u32);

struct Number;

#[entity]
impl Number {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Value(value))
    }

    fn build_without_value() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    fn build_with_additional_component(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value(value))
            .with(String::from("other"))
    }

    #[run]
    fn collect(
        value: &Value,
        mut collector: SingleMut<'_, StreamCollector>,
        _filter: Filter<With<String>>,
    ) {
        collector.0.push(value.0);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

struct OtherNumber;

#[entity]
impl OtherNumber {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Value(value))
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn iterate_using_filter() {
    App::new()
        .with_entity(StreamCollector::build())
        .with_entity(Number::build(1))
        .with_entity(OtherNumber::build(10))
        .with_entity(Number::build(2))
        .with_entity(Number::build_without_value())
        .with_entity(Number::build_with_additional_component(3))
        .updated()
        .assert::<With<StreamCollector>>(1, |e| e.has(|c: &StreamCollector| assert_eq!(c.0, [3])));
}
