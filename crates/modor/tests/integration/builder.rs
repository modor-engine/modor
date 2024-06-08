use modor_derive::Builder;

#[modor::test]
fn use_builder_methods() {
    let built = Test::default()
        .with_value(42)
        .with_closure(|vec| vec.push(10));
    assert_eq!(built.value, 42);
    assert_eq!(built.closure, [10]);
    assert_eq!(built.ignored, 0);
}

#[derive(Default, Builder)]
struct Test {
    #[builder(form(value))]
    value: u32,
    #[builder(form(closure))]
    closure: Vec<i64>,
    ignored: u8,
}
