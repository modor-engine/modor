use modor::testing::TestApp;
use modor::{entity, App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_physics::{Position, Scale, Shape};

struct Object;

#[entity]
impl Object {
    fn build_default() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(-0.25, 0.25))
            .with(Scale::xy(0.4, 0.25))
            .with(ShapeColor(Color::RED))
    }

    fn build_rectangle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0.25, 0.25))
            .with(Scale::xy(0.4, 0.25))
            .with(ShapeColor(Color::GREEN))
            .with(Shape::Rectangle2D)
    }

    fn build_circle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(-0.25, -0.25))
            .with(Scale::xy(0.4, 0.25))
            .with(ShapeColor(Color::BLUE))
            .with(Shape::Circle2D)
    }
}

#[test]
fn display_shapes() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_default())
        .with_entity(Object::build_circle())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/shapes.png");
}
