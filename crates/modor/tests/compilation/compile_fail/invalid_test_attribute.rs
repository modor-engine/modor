#[macro_use]
extern crate modor;

fn main() {}

#[modor_test]
fn ok1() {}

#[modor_test()]
fn ok2() {}

#[modor_test(disabled(wasm))]
fn ok3() {}

#[modor_test(disabled(windows, linux, macos, wasm, android))]
fn ok4() {}

#[modor_test(windows = "false")]
//~^ error: expected syntax: `#[modor_test]` or `#[modor_test(disabled(platform1, ...))]`
fn nok2() {}

#[modor_test(enabled(windows))]
//~^ error: expected syntax: `#[modor_test]` or `#[modor_test(disabled(platform1, ...))]`
fn nok3() {}

#[modor_test(disabled::none)]
//~^ error: expected syntax: `#[modor_test]` or `#[modor_test(disabled(platform1, ...))]`
fn nok4() {}

#[modor_test(disabled)]
//~^ error: expected syntax: `#[modor_test]` or `#[modor_test(disabled(platform1, ...))]`
fn nok5() {}

#[modor_test(disabled(platform))]
//~^ error: allowed platforms are ["android", "linux", "macos", "wasm", "windows"]
fn nok6() {}

#[modor_test(disabled(windows), enabled(linux))]
//~^ error: max one argument is allowed
fn nok7() {}
