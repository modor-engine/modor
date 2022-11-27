use modor::{App, Built, Entity, EntityBuilder, Filter, Query, With, World};
use modor_graphics::{
    testing, Capture, Color, GraphicsModule, Mesh2D, Resource, ResourceState, SurfaceSize, Texture,
    TexturePart, TextureRef,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::thread;
use std::time::Duration;

use crate::PathTextureRef;
use log::LevelFilter;

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
        texture_ref: Option<impl TextureRef>,
        color: Color,
        texture_color: Color,
        texture_part: TexturePart,
    ) -> impl Built<Self> {
        let mut mesh = Mesh2D::rectangle()
            .with_color(color)
            .with_texture_color(texture_color);
        if let Some(texture_id) = texture_ref {
            mesh = mesh.with_texture(texture_id);
        }
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.4, 0.3)),
            )
            .with(mesh.with_texture_part(texture_part))
    }
}

struct TextureRemover;

#[singleton]
impl TextureRemover {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn update(textures: Query<'_, (Entity<'_>, Filter<With<Texture>>)>, mut world: World<'_>) {
        for (texture, _) in textures.iter() {
            world.delete_entity(texture.id());
        }
    }
}

#[test]
fn display_shapes() {
    App::new()
        .with_log_level(LevelFilter::Info)
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
            None as Option<PathTextureRef>,
            Color::INVISIBLE,
            Color::WHITE,
            TexturePart::default(),
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
        .with_entity(Texture::build(PathTextureRef::OpaquePixelated))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(-0.25, 0.25),
            None as Option<PathTextureRef>,
            Color::GREEN,
            Color::BLUE,
            TexturePart::default(),
        ))
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(0.25, 0.25),
            Some(PathTextureRef::OpaquePixelated),
            Color::GREEN,
            Color::BLUE,
            TexturePart::default(),
        ))
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(-0.25, -0.25),
            Some(PathTextureRef::OpaqueSmooth),
            Color::GREEN,
            Color::BLUE,
            TexturePart::default(),
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

#[test]
fn update_attached_texture() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(PathTextureRef::OpaquePixelated))
        .with_entity(Texture::build(PathTextureRef::TransparentPixelated))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(-0.25, 0.25),
            Some(PathTextureRef::OpaquePixelated),
            Color::GREEN,
            Color::BLUE,
            TexturePart::default(),
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/initial_texture.png")
        })
        .with_update::<(), _>(|m: &mut Mesh2D| {
            m.attach_texture(PathTextureRef::TransparentPixelated);
        })
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/modified_texture.png")
        })
        .with_update::<(), _>(|m: &mut Mesh2D| {
            m.detach_texture();
        })
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/removed_texture.png")
        });
}

#[test]
fn configure_texture_part() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(PathTextureRef::Colored))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(Object::build_styled_rectangle(
            Vec2::new(-0.25, 0.25),
            Some(PathTextureRef::Colored),
            Color::WHITE,
            Color::WHITE,
            TexturePart::default()
                .with_position(Vec2::new(0., 0.5))
                .with_size(Vec2::new(0.5, 0.5)),
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_part.png")
        });
}
