use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder, SingleMut};
use modor_graphics::{
    testing, BackgroundColor, Capture, Color, GraphicsModule, Mesh, SurfaceSize,
    UpdateCaptureBufferAction,
};
use modor_math::Vec3;
use modor_physics::{RelativeTransform, Transform};

struct CaptureResizer {
    new_size: SurfaceSize,
}

#[entity]
impl CaptureResizer {
    fn build(new_size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self { new_size })
    }

    #[run_after(UpdateCaptureBufferAction)]
    fn run(&self, mut capture: SingleMut<'_, Capture>) {
        capture.set_size(self.new_size);
    }
}

struct Rectangle;

#[entity]
impl Rectangle {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform::new().with_size(Vec3::ONE * 0.9))
            .with(Mesh::rectangle().with_color(Color::RED))
            .with_child(Quarter::build())
    }
}

struct Quarter;

#[entity]
impl Quarter {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform::default())
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::xyz(0.25, 0.25, 1.))
                    .with_size(Vec3::ONE * 0.5),
            )
            .with(Mesh::rectangle().with_color(Color::MAROON))
    }
}

#[test]
fn resize_capture_smaller() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
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
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
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
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
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
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .with_entity(Rectangle::build())
        .with_entity(CaptureResizer::build(SurfaceSize::new(200, 300)))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_initial_size.png");
    app.update();
    testing::assert_capture(&app, "tests/expected/capture_vertical.png");
}
