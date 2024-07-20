#[modor::test(cases(one = "1", two = "2"))]
fn check_test_cases(value: u32) {
    assert!(value == 1 || value == 2);
}

#[modor::test(disabled(wasm))]
fn check_test_disabled_platforms() {
    // do nothing, just ensure it compiles
}
