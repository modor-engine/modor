use modor::testing::TestApp;
use modor::{App, Built, Entity, EntityBuilder, World};
use modor_graphics::{testing, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_physics::{Position, Scale, Shape};

struct Object;

#[entity]
impl Object {
    fn build_rectangle(position: Position, color: Color) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(position)
            .with(Scale::xy(0.2, 0.2))
            .with(ShapeColor(color))
    }

    fn build_circle(position: Position, color: Color) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(position)
            .with(Scale::xy(0.2, 0.2))
            .with(Shape::Circle2D)
            .with(ShapeColor(color))
    }

    #[run]
    fn clean_up(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

#[test]
fn display_transparent_and_opaque_shapes_ordered() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_rectangle(
            Position::xyz(0., 0., 0.),
            Color::rgb(0., 0., 1.),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.05, 0.05, 1.),
            Color::rgba(1., 1., 1., 0.8),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.1, 0.1, 2.),
            Color::rgba(0., 1., 0., 0.2),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/transparency_with_opaque.png");
}

#[test]
fn display_transparent_and_opaque_shapes_unordered() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.1, 0.1, 2.),
            Color::rgba(0., 1., 0., 0.2),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0., 0., 0.),
            Color::rgb(0., 0., 1.),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.05, 0.05, 1.),
            Color::rgba(1., 1., 1., 0.8),
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/transparency_with_opaque.png");
}

#[test]
fn display_different_transparent_shapes() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_circle(
            Position::xyz(0.15, 0.15, 3.),
            Color::rgba(1., 0., 0., 0.5),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.1, 0.1, 2.),
            Color::rgba(0., 1., 0., 0.5),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0., 0., 0.),
            Color::rgba(0., 0., 1., 0.5),
        ))
        .with_entity(Object::build_circle(
            Position::xyz(0.05, 0.05, 1.),
            Color::rgba(1., 1., 1., 0.5),
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/transparency_with_multiple_shapes.png");
}

#[test]
fn hide_shape_after_deletion() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.1, 0.1, 2.),
            Color::rgba(0., 1., 0., 0.2),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0., 0., 0.),
            Color::rgb(0., 0., 1.),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.05, 0.05, 1.),
            Color::rgba(1., 1., 1., 0.8),
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/transparency_with_opaque.png");
    let mut app: TestApp = App::from(app)
        .with_entity(Object::build_rectangle(
            Position::xyz(0., 0., 1.),
            Color::rgba(1., 1., 0., 0.5),
        ))
        .with_entity(Object::build_rectangle(
            Position::xyz(0.25, 0.25, 2.),
            Color::rgba(1., 1., 0., 0.5),
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/transparency_cleaned_up.png");
}
