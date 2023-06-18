use modor::{App, With};
use modor_graphics::{testing, BackgroundColor, Capture, Color, GraphicsModule, SurfaceSize};
use std::fs;
use std::path::Path;

#[modor_test(disabled(macos, android, wasm))]
fn capture_save_if_not_existing() {
    let written_file_path = Path::new("tests/expected/testing_write.png");
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::from(Color::GREEN))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/testing.png")
        })
        .assert::<With<Capture>>(1, |e| testing::assert_capture(e, written_file_path));
    let file_exists = written_file_path.exists();
    let _result = fs::remove_file(written_file_path);
    assert!(file_exists);
}

#[modor_test(disabled(macos, android, wasm))]
#[should_panic]
fn fail_if_no_capture() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::from(Color::GREEN))
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/testing_no_capture.png")
        });
}

#[modor_test(disabled(macos, android, wasm))]
#[should_panic]
fn fail_testing_if_captures_different() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::from(Color::RED))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/testing.png")
        });
}
