use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{testing, Capture, Color, GraphicsModule, Mesh2D, SurfaceSize};
use modor_math::Vec2;
use modor_physics::{RelativeTransform2D, Transform2D};

fn character(position: Vec2, size: Vec2, angle: f32) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(size)
                .with_rotation(angle.to_radians()),
        )
        .with_child(character_head())
        .with_child(character_body())
}

fn character_head() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::default())
        .with(
            RelativeTransform2D::new()
                .with_position(Vec2::new(0., 0.4))
                .with_size(Vec2::new(0.2, 0.2))
                .with_rotation(0.),
        )
        .with(Mesh2D::rectangle().with_color(Color::BLUE))
}

fn character_body() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::default())
        .with(
            RelativeTransform2D::new()
                .with_position(Vec2::new(0., -0.1))
                .with_size(Vec2::new(0.4, 0.8))
                .with_rotation(0.),
        )
        .with(Mesh2D::rectangle().with_color(Color::GREEN))
}

fn center() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::ONE * 0.05))
        .with(Mesh2D::ellipse())
}

#[test]
fn display_hierarchy() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(center())
        .with_entity(character(Vec2::new(0.25, 0.25), Vec2::new(0.5, 0.5), -20.))
        .with_entity(character(Vec2::new(-0.1, -0.1), Vec2::new(0.3, 0.1), 0.))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/hierarchy.png")
        });
}
