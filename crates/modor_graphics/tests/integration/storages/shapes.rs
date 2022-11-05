use modor::{App, Built, Entity, EntityBuilder, Query, With, World};
use modor_graphics::{
    testing, Capture, Color, GraphicsModule, Mesh2D, SurfaceSize, Texture, TextureConfig,
    TextureState,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::thread;
use std::time::Duration;

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

    fn build_styled_rectangle(
        position: Vec2,
        texture_id: Option<usize>,
        color: Color,
        texture_color: Color,
    ) -> impl Built<Self> {
        let mut mesh = Mesh2D::rectangle()
            .with_color(color)
            .with_texture_color(texture_color);
        if let Some(texture_id) = texture_id {
            mesh = mesh.with_texture(texture_id);
        }
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.4, 0.3)),
            )
            .with(mesh)
    }
}

struct TextureRemover;

#[singleton]
impl TextureRemover {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn update(textures: Query<'_, Entity<'_>, With<Texture>>, mut world: World<'_>) {
        for texture in textures.iter() {
            world.delete_entity(texture.id());
        }
    }
}

#[test]
fn display_shapes() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_ellipse())
        .with_entity(Object::build_rectangle())
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/shapes.png")
        });
}

#[test]
fn display_invisible_shape() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Object::build_styled_rectangle(
            Vec2::ZERO,
            None,
            Color::INVISIBLE,
            Color::WHITE,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/invisible_shape.png")
        });
}

#[test]
fn attach_texture_with_color() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(
            TextureConfig::from_path(0_usize, "../tests/assets/opaque-texture.png")
                .with_smooth(false),
        ))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(-0.25, 0.25),
            None,
            Color::GREEN,
            Color::BLUE,
        ))
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(0.25, 0.25),
            Some(0),
            Color::GREEN,
            Color::BLUE,
        ))
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(-0.25, -0.25),
            Some(1),
            Color::GREEN,
            Color::BLUE,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_with_color.png")
        })
        .with_entity(TextureRemover::build())
        .updated()
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/removed_texture_with_color.png")
        });
}