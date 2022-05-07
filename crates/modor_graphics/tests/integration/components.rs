use modor::testing::TestApp;
use modor::{entity, App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_physics::{Position, Scale};

struct Rectangle;

#[entity]
impl Rectangle {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., 0.))
            .with(Scale::xy(0.9, 0.9))
            .with(ShapeColor(Color::RED))
    }

    #[run]
    fn update_shape_color(shape: &mut ShapeColor) {
        shape.g = shape.r;
    }
}

#[test]
fn update_components() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Rectangle::build())
        .into();
    app.update();
    app.update();
    testing::assert_capture(&app, "tests/expected/components.png");
}