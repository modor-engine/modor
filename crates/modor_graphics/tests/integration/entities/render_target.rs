use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{
    testing, BackgroundColor, Capture, Color, GraphicsModule, Mesh2D, SurfaceSize,
};
use modor_math::Vec2;
use modor_physics::{RelativeTransform2D, Transform2D};

fn rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::ONE * 0.9))
        .with(Mesh2D::rectangle().with_color(Color::RED))
        .with_child(rectangle_quarter())
}

fn rectangle_quarter() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::default())
        .with(
            RelativeTransform2D::new()
                .with_position(Vec2::new(0.25, 0.25))
                .with_size(Vec2::ONE * 0.5),
        )
        .with(Mesh2D::rectangle().with_color(Color::MAROON).with_z(1.))
}

#[modor_test(disabled(macos, android, wasm))]
fn resize_capture() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::from(Color::GREEN))
        .with_entity(rectangle())
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/capture_initial_size.png")
        })
        .with_update::<(), _>(|c: &mut Capture| c.set_size(SurfaceSize::new(100, 50)))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/capture_smaller.png")
        })
        .with_update::<(), _>(|c: &mut Capture| c.set_size(SurfaceSize::new(400, 300)))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/capture_bigger.png")
        })
        .with_update::<(), _>(|c: &mut Capture| c.set_size(SurfaceSize::new(0, 0)))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/capture_bigger.png")
        })
        .with_update::<(), _>(|c: &mut Capture| c.set_size(SurfaceSize::new(200, 300)))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/capture_vertical.png")
        });
}
