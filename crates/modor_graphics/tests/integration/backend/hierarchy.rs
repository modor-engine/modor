use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_graphics::{testing, Color, GraphicsModule, Mesh, SurfaceSize};
use modor_math::{Quat, Vec3};
use modor_physics::{RelativeTransform, Transform};

struct Character;

#[entity]
impl Character {
    fn build(position: Vec3, size: Vec3, angle: f32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform::new()
                    .with_position(position)
                    .with_size(size)
                    .with_rotation(Quat::from_z(angle.to_radians())),
            )
            .with_child(CharacterHead::build())
            .with_child(CharacterBody::build())
    }
}

struct CharacterHead;

#[entity]
impl CharacterHead {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform::default())
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::xy(0., 0.4))
                    .with_size(Vec3::xy(0.2, 0.2))
                    .with_rotation(Quat::ZERO),
            )
            .with(Mesh::rectangle().with_color(Color::BLUE))
    }
}

struct CharacterBody;

#[entity]
impl CharacterBody {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform::default())
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::xy(0., -0.1))
                    .with_size(Vec3::xy(0.4, 0.8))
                    .with_rotation(Quat::ZERO),
            )
            .with(Mesh::rectangle().with_color(Color::GREEN))
    }
}

struct Center;

#[entity]
impl Center {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform::new().with_size(Vec3::ONE * 0.05))
            .with(Mesh::ellipse())
    }
}

#[test]
fn display_hierarchy() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Center::build())
        .with_entity(Character::build(
            Vec3::xy(0.25, 0.25),
            Vec3::xy(0.5, 0.5),
            20.,
        ))
        .with_entity(Character::build(
            Vec3::xy(-0.1, -0.1),
            Vec3::xy(0.3, 0.1),
            0.,
        ))
        .into();
    app.update();
    testing::assert_capture(&app, "tests/expected/hierarchy.png");
}
