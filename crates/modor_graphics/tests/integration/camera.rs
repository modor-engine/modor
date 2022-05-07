use modor::testing::TestApp;
use modor::{entity, App, Built, EntityBuilder};
use modor_graphics::{testing, Camera2D, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_physics::{Position, Scale, Shape};

struct Object;

#[entity]
impl Object {
    fn build_rectangle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0.25, 0.25))
            .with(Scale::xy(0.4, 0.25))
            .with(ShapeColor(Color::GREEN))
            .with(Shape::Rectangle2D)
    }

    fn build_circle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(-0.25, 0.25))
            .with(Scale::xy(0.4, 0.25))
            .with(ShapeColor(Color::BLUE))
            .with(Shape::Circle2D)
    }
}

#[test]
fn add_camera_with_horizontal_surface() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Camera2D::build(Position::xy(0.5, 0.5), Scale::xy(3., 3.)))
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
        .with_entity(Camera2D::build(Position::xy(0.5, 0.5), Scale::xy(3., 3.)))
        .with_entity(Object::build_circle())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/camera_vertical.png");
}
