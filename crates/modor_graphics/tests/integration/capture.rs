use modor::testing::TestApp;
use modor::{entity, App, Built, EntityBuilder, SingleMut};
use modor_graphics::{
    testing, BackgroundColor, Capture, Color, GraphicsModule, ShapeColor, SurfaceSize,
    UpdateCaptureBuffer,
};
use modor_physics::{Position, Scale};

struct CaptureResizer {
    new_size: SurfaceSize,
}

#[entity]
impl CaptureResizer {
    fn build(new_size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self { new_size })
    }

    #[run_after(UpdateCaptureBuffer)]
    fn run(&self, mut capture: SingleMut<'_, Capture>) {
        capture.set_size(self.new_size);
    }
}

struct Rectangle;

#[entity]
impl Rectangle {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., 0.))
            .with(Scale::xy(0.9, 0.9))
            .with(ShapeColor(Color::RED))
    }
}

#[test]
fn resize_capture_smaller() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .with_entity(Rectangle::build())
        .with_entity(CaptureResizer::build(SurfaceSize::new(100, 50)))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_initial_size.png");
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_smaller.png");
}

#[test]
fn resize_capture_bigger() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .with_entity(Rectangle::build())
        .with_entity(CaptureResizer::build(SurfaceSize::new(400, 300)))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_initial_size.png");
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_bigger.png");
}

#[test]
fn resize_capture_to_zero() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .with_entity(Rectangle::build())
        .with_entity(CaptureResizer::build(SurfaceSize::new(0, 0)))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_initial_size.png");
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_zero.png");
}

#[test]
fn resize_capture_vertically() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless())
        .with_entity(Capture::build(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .with_entity(Rectangle::build())
        .with_entity(CaptureResizer::build(SurfaceSize::new(200, 300)))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_initial_size.png");
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_vertical.png");
}
