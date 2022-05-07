use modor::testing::TestApp;
use modor::App;
use modor_graphics::{testing, BackgroundColor, Color, GraphicsModule, SurfaceSize};
use std::fs;
use std::path::Path;

#[test]
fn capture_save_if_not_existing() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/testing.png");
    let written_file_path = Path::new("tests/expected/testing_write.png");
    testing::assert_capture(&app, written_file_path);
    let file_exists = written_file_path.exists();
    let _result = fs::remove_file(written_file_path);
    assert!(file_exists);
}

#[test]
#[should_panic]
fn fail_if_no_capture() {
    let app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .into();
    testing::assert_capture(&app, "tests/expected/testing_no_capture.png");
}

#[test]
#[should_panic]
fn fail_testing_if_captures_different() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/testing.png");
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::RED))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/testing.png");
}
