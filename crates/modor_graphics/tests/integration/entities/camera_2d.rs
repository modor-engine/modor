use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{testing, Camera2D, Capture, Color, GraphicsModule, Mesh2D, SurfaceSize};
use modor_math::Vec2;
use modor_physics::Transform2D;

fn center() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::new(0.5, 0.5))
                .with_size(Vec2::ONE * 0.1),
        )
        .with(Mesh2D::ellipse().with_z(1.))
}

fn rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::new(0.25, 0.25))
                .with_size(Vec2::new(0.4, 0.25)),
        )
        .with(Mesh2D::rectangle().with_color(Color::GREEN))
}

fn ellipse() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::new(-0.25, 0.25))
                .with_size(Vec2::new(0.4, 0.25)),
        )
        .with(Mesh2D::ellipse().with_color(Color::BLUE))
}

#[modor_test(disabled(macos, android, wasm))]
fn add_camera_with_horizontal_surface() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Camera2D::build(Vec2::new(0.5, 0.5), Vec2::new(3., 3.)))
        .with_entity(center())
        .with_entity(ellipse())
        .with_entity(rectangle())
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/camera_horizontal.png")
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn add_camera_with_vertical_surface() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(200, 300)))
        .with_entity(Camera2D::build(Vec2::new(0.5, 0.5), Vec2::new(3., 3.)))
        .with_entity(center())
        .with_entity(ellipse())
        .with_entity(rectangle())
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/camera_vertical.png")
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn add_rotated_camera() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Camera2D::build_rotated(
            Vec2::new(0.5, 0.5),
            Vec2::new(3., 3.),
            -45_f32.to_radians(),
        ))
        .with_entity(center())
        .with_entity(ellipse())
        .with_entity(rectangle())
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/camera_rotated.png")
        });
}
