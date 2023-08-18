#[macro_use]
extern crate modor;

fn main() {}

#[modor_test]
fn valid_no_arg() {}

#[modor_test()]
fn valid_empty_args() {}

#[modor_test(disabled(wasm))]
fn valid_disabled_platform() {}

#[modor_test(disabled(windows, linux, macos, wasm, android))]
fn valid_all_disabled_platforms() {}

#[modor_test(enabled(windows))]
//~^ error: Unknown field: `enabled`
fn invalid_unknown_field() {}

#[modor_test(disabled(windows), enabled(linux))]
//~^ error: Unknown field: `enabled`
fn invalid_unknown_field_with_known_field() {}

#[modor_test(disabled = "false")]
//~^ error: Unexpected literal type `string`
fn invalid_disabled_format() {}

#[modor_test(disabled::none)]
//~^ error: Unknown field: `disabled::none`
fn invalid_disabled_path() {}

#[modor_test(disabled)]
//~^ error: Unexpected meta-item format `word`
fn invalid_empty_disabled() {}

#[modor_test(disabled(windows = "false"))]
//~^ error: Unexpected literal type `non-word`
fn invalid_disabled_platform_format() {}

#[modor_test(disabled(platform))]
//~^ error: allowed platforms are ["android", "linux", "macos", "wasm", "windows"]
fn invalid_disabled_unknown_platform() {}

#[modor_test(disabled(windows::platform))]
//~^ error: allowed platforms are ["android", "linux", "macos", "wasm", "windows"]
fn invalid_disabled_unknown_path_platform() {}
