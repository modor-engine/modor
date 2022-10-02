use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Camera2D, Color, GraphicsModule, Mesh2D, SurfaceSize};
use modor_math::Vec2;
use modor_physics::Transform2D;

struct Object;

#[entity]
impl Object {
    fn build_center() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(Vec2::new(0.5, 0.5))
                    .with_size(Vec2::ONE * 0.1),
            )
            .with(Mesh2D::ellipse().with_z(1.))
    }

    fn build_rectangle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(Vec2::new(0.25, 0.25))
                    .with_size(Vec2::new(0.4, 0.25)),
            )
            .with(Mesh2D::rectangle().with_color(Color::GREEN))
    }

    fn build_ellipse() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(Vec2::new(-0.25, 0.25))
                    .with_size(Vec2::new(0.4, 0.25)),
            )
            .with(Mesh2D::ellipse().with_color(Color::BLUE))
    }
}

#[test]
fn add_camera_with_horizontal_surface() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Camera2D::build(Vec2::new(0.5, 0.5), Vec2::new(3., 3.)))
        .with_entity(Object::build_center())
        .with_entity(Object::build_ellipse())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/camera_horizontal.png");
}

#[test]
fn add_camera_with_vertical_surface() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(200, 300)))
        .with_entity(Camera2D::build(Vec2::new(0.5, 0.5), Vec2::new(3., 3.)))
        .with_entity(Object::build_center())
        .with_entity(Object::build_ellipse())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/camera_vertical.png");
}

#[test]
fn add_rotated_camera() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Camera2D::build_rotated(
            Vec2::new(0.5, 0.5),
            Vec2::new(3., 3.),
            -45_f32.to_radians(),
        ))
        .with_entity(Object::build_center())
        .with_entity(Object::build_ellipse())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/camera_rotated.png");
}
