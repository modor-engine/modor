use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Camera2D, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_math::{Quat, Vec3};
use modor_physics::{Position, Rotation, Shape, Size};

struct Object;

#[entity]
impl Object {
    fn build_center() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xyz(0.5, 0.5, 1.)))
            .with(Size::from(Vec3::xy(0.1, 0.1)))
            .with(ShapeColor::from(Color::WHITE))
            .with(Shape::Circle2D)
    }

    fn build_rectangle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xy(0.25, 0.25)))
            .with(Size::from(Vec3::xy(0.4, 0.25)))
            .with(ShapeColor::from(Color::GREEN))
            .with(Shape::Rectangle2D)
    }

    fn build_circle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xy(-0.25, 0.25)))
            .with(Size::from(Vec3::xy(0.4, 0.25)))
            .with(ShapeColor::from(Color::BLUE))
            .with(Shape::Circle2D)
    }
}

#[test]
fn add_camera_with_horizontal_surface() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Camera2D::build(
            Position::from(Vec3::xy(0.5, 0.5)),
            Size::from(Vec3::xy(3., 3.)),
        ))
        .with_entity(Object::build_center())
        .with_entity(Object::build_circle())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/camera_horizontal.png");
}

#[test]
fn add_camera_with_vertical_surface() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(200, 300)))
        .with_entity(Camera2D::build(
            Position::from(Vec3::xy(0.5, 0.5)),
            Size::from(Vec3::xy(3., 3.)),
        ))
        .with_entity(Object::build_center())
        .with_entity(Object::build_circle())
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
            Position::from(Vec3::xy(0.5, 0.5)),
            Size::from(Vec3::xy(3., 3.)),
            Rotation::from(Quat::from_z(20_f32.to_radians())),
        ))
        .with_entity(Object::build_center())
        .with_entity(Object::build_circle())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/camera_rotated.png");
}
