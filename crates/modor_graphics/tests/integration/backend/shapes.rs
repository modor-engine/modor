use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, Shape, ShapeColor, SurfaceSize};
use modor_math::Vec3;
use modor_physics::Transform;

struct Object;

#[entity]
impl Object {
    fn build_default() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform::new()
                    .with_position(Vec3::xy(-0.25, 0.25))
                    .with_size(Vec3::xy(0.4, 0.25)),
            )
            .with(ShapeColor::from(Color::RED))
    }

    fn build_rectangle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform::new()
                    .with_position(Vec3::xy(0.25, 0.25))
                    .with_size(Vec3::xy(0.4, 0.25)),
            )
            .with(ShapeColor::from(Color::GREEN))
            .with(Shape::Rectangle)
    }

    fn build_circle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform::new()
                    .with_position(Vec3::xy(-0.25, -0.25))
                    .with_size(Vec3::xy(0.4, 0.25)),
            )
            .with(ShapeColor::from(Color::BLUE))
            .with(Shape::Circle)
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
