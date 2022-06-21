use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor_math::{Quat, Vec3};
use modor_physics::{
    Position, RelativePosition, RelativeRotation, RelativeSize, Rotation, Shape, Size,
};

struct Character;

#[entity]
impl Character {
    fn build(position: Position, size: Size, angle: f32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(position)
            .with(size)
            .with(Rotation::from(Quat::from_z(angle.to_radians())))
            .with_child(CharacterHead::build())
            .with_child(CharacterBody::build())
    }
}

struct CharacterHead;

#[entity]
impl CharacterHead {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(RelativePosition::from(Vec3::xy(0., 0.4)))
            .with(RelativeSize::from(Vec3::xy(0.2, 0.2)))
            .with(RelativeRotation::from(Quat::ZERO))
            .with(Position::from(Vec3::ZERO))
            .with(Size::from(Vec3::ZERO))
            .with(Rotation::from(Quat::ZERO))
            .with(ShapeColor::from(Color::BLUE))
    }
}

struct CharacterBody;

#[entity]
impl CharacterBody {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(RelativePosition::from(Vec3::xy(0., -0.1)))
            .with(RelativeSize::from(Vec3::xy(0.4, 0.8)))
            .with(RelativeRotation::from(Quat::from_z(0.)))
            .with(Position::from(Vec3::ZERO))
            .with(Size::from(Vec3::ZERO))
            .with(Rotation::from(Quat::ZERO))
            .with(ShapeColor::from(Color::GREEN))
    }
}

struct Center;

#[entity]
impl Center {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xy(0., 0.)))
            .with(Size::from(Vec3::xy(0.05, 0.05)))
            .with(ShapeColor::from(Color::WHITE))
            .with(Shape::Circle2D)
    }
}

#[test]
fn display_hierarchy() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Center::build())
        .with_entity(Character::build(
            Position::from(Vec3::xy(0.25, 0.25)),
            Size::from(Vec3::xy(0.5, 0.5)),
            20.,
        ))
        .with_entity(Character::build(
            Position::from(Vec3::xy(-0.1, -0.1)),
            Size::from(Vec3::xy(0.3, 0.1)),
            0.,
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/hierarchy.png");
}
