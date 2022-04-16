use modor::testing::TestApp;
use modor::App;
use modor_graphics::{testing, BackgroundColor, Capture, Color, GraphicsModule, SurfaceSize};

#[test]
#[should_panic]
fn fail_if_no_capture() {
    let app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .into();
    testing::assert_capture(&app, "tests/expected/testing_no_capture.png");
}

#[test]
#[should_panic]
fn fail_testing_if_captures_different() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/testing.png");
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::RED))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/testing.png");
}
