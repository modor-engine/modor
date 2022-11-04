use modor::{App, Built, Entity, EntityBuilder, With, World};
use modor_graphics::{
    testing, Capture, Color, GraphicsModule, Mesh2D, SurfaceSize, Texture, TextureConfig,
    TextureState,
};
use modor_math::{Vec2, Vec3};
use modor_physics::Transform2D;
use std::thread;
use std::time::Duration;

struct Object;

#[entity]
impl Object {
    fn build_rectangle(position: Vec3, color: Color, texture_id: usize) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position.xy())
                    .with_size(Vec2::ONE * 0.2),
            )
            .with(
                Mesh2D::rectangle()
                    .with_texture(texture_id)
                    .with_texture_color(color)
                    .with_z(position.z),
            )
    }

    fn build_ellipse(position: Vec3, color: Color, texture_id: usize) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position.xy())
                    .with_size(Vec2::ONE * 0.2),
            )
            .with(
                Mesh2D::ellipse()
                    .with_texture(texture_id)
                    .with_texture_color(color)
                    .with_z(position.z),
            )
    }

    #[run]
    fn clean_up(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

#[test]
fn display_transparent_and_opaque_shapes_ordered() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(
            TextureConfig::from_path(0_usize, "../tests/assets/transparent-texture.png")
                .with_smooth(false),
        ))
        .with_entity(Texture::build(
            TextureConfig::from_path(1_usize, "../tests/assets/opaque-texture.png")
                .with_smooth(false),
        ))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .with_entity(Object::build_rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            1,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            1,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_transparency_with_opaque.png")
        });
}

#[test]
fn display_transparent_and_opaque_shapes_unordered() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(
            TextureConfig::from_path(0_usize, "../tests/assets/transparent-texture.png")
                .with_smooth(false),
        ))
        .with_entity(Texture::build(
            TextureConfig::from_path(1_usize, "../tests/assets/opaque-texture.png")
                .with_smooth(false),
        ))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .with_entity(Object::build_rectangle(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            1,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            1,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            0,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_transparency_with_opaque.png")
        });
}

#[test]
fn display_different_transparent_shapes() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(
            TextureConfig::from_path(0_usize, "../tests/assets/transparent-texture.png")
                .with_smooth(false),
        ))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .with_entity(Object::build_ellipse(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            0,
        ))
        .with_entity(Object::build_ellipse(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            0,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(
                e,
                "tests/expected/texture_transparency_with_multiple_shapes.png",
            )
        });
}

#[test]
fn hide_shape_after_deletion() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(
            TextureConfig::from_path(0_usize, "../tests/assets/transparent-texture.png")
                .with_smooth(false),
        ))
        .with_entity(Texture::build(
            TextureConfig::from_path(1_usize, "../tests/assets/opaque-texture.png")
                .with_smooth(false),
        ))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .with_entity(Object::build_rectangle(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            1,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            1,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            0,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_transparency_with_opaque.png")
        })
        .with_entity(Object::build_rectangle(
            Vec3::new(0., 0., 1.),
            Color::rgb(1., 1., 0.),
            0,
        ))
        .with_entity(Object::build_rectangle(
            Vec3::new(0.25, 0.25, 2.),
            Color::rgb(1., 1., 0.),
            0,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_transparency_cleaned_up.png")
        });
}
