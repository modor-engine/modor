use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, Mesh2D, SurfaceSize};
use modor_math::Vec2;
use modor_physics::Transform2D;

struct Object;

#[entity]
impl Object {
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
                    .with_position(Vec2::new(-0.25, -0.25))
                    .with_size(Vec2::new(0.4, 0.25)),
            )
            .with(Mesh2D::ellipse().with_color(Color::BLUE))
    }
}

#[test]
fn display_shapes() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_ellipse())
        .with_entity(Object::build_rectangle())
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/shapes.png");
}
