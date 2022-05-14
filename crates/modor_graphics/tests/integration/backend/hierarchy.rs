use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_physics::{Position, Scale, Shape};

struct Character;

#[entity]
impl Character {
    fn build(position: Position, scale: Scale) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(position)
            .with(scale)
            .with_child(CharacterHead::build())
            .with_child(CharacterBody::build())
    }
}

struct CharacterHead;

#[entity]
impl CharacterHead {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., 0.4))
            .with(Scale::xy(0.2, 0.2))
            .with(ShapeColor(Color::BLUE))
    }
}

struct CharacterBody;

#[entity]
impl CharacterBody {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., -0.1))
            .with(Scale::xy(0.4, 0.8))
            .with(ShapeColor(Color::GREEN))
    }
}

struct Center;

#[entity]
impl Center {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::xy(0., 0.))
            .with(Scale::xy(0.05, 0.05))
            .with(ShapeColor(Color::WHITE))
            .with(Shape::Circle2D)
    }
}

#[test]
fn display_hierarchy() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Center::build())
        .with_entity(Character::build(
            Position::xy(0.25, 0.25),
            Scale::xy(0.5, 0.5),
        ))
        .with_entity(Character::build(
            Position::xy(-0.1, -0.1),
            Scale::xy(0.3, 0.1),
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/hierarchy.png");
}
