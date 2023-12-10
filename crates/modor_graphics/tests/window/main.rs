#![allow(clippy::missing_panics_doc, clippy::unwrap_used)]

#[macro_use]
extern crate modor;

#[modor_test(disabled(windows, macos, android, wasm))]
pub fn run_window_tests() {
    let mut context = modor_graphics::testing::TestRunnerContext::default();
    window::run_window_tests(&mut context);
    input::run_window_tests(&mut context);
}

pub mod input;
pub mod window;
