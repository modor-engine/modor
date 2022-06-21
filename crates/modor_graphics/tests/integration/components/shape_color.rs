use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_math::Vec3;
use modor_physics::{Position, Size};

struct Rectangle;

#[entity]
impl Rectangle {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xy(0., 0.)))
            .with(Size::from(Vec3::xy(0.9, 0.9)))
            .with(ShapeColor::from(Color::RED))
    }

    #[run]
    fn update_shape_color(shape_color: &mut ShapeColor) {
        shape_color.g = shape_color.r;
        let color: Color = (*shape_color).into();
        assert_abs_diff_eq!(color.r, 1.);
        assert_abs_diff_eq!(color.g, 1.);
        assert_abs_diff_eq!(color.b, 0.);
    }
}

#[test]
fn update_shape_color() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Rectangle::build())
        .into();
    app.update();
    app.update();
    testing::assert_capture(&app, "tests/expected/components.png");
}
