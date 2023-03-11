use crate::{AutoRemoved, PathTextureRef};
use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{
    testing, Capture, Color, GraphicsModule, Mesh2D, Resource, ResourceState, SurfaceSize, Texture,
    TextureRef,
};
use modor_math::{Vec2, Vec3};
use modor_physics::Transform2D;
use std::thread;
use std::time::Duration;

fn rectangle(position: Vec3, color: Color, texture_ref: impl TextureRef) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position.xy())
                .with_size(Vec2::ONE * 0.2),
        )
        .with(
            Mesh2D::rectangle()
                .with_texture(texture_ref)
                .with_texture_color(color)
                .with_z(position.z),
        )
}

fn ellipse(position: Vec3, color: Color, texture_ref: impl TextureRef) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position.xy())
                .with_size(Vec2::ONE * 0.2),
        )
        .with(
            Mesh2D::ellipse()
                .with_texture(texture_ref)
                .with_texture_color(color)
                .with_z(position.z),
        )
}

#[test]
fn display_transparent_and_opaque_shapes_ordered() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::new(PathTextureRef::TransparentPixelated))
        .with_entity(Texture::new(PathTextureRef::OpaquePixelated))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            PathTextureRef::OpaquePixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            PathTextureRef::OpaquePixelated,
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
        .with_entity(Texture::new(PathTextureRef::TransparentPixelated))
        .with_entity(Texture::new(PathTextureRef::OpaquePixelated))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(rectangle(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            PathTextureRef::OpaquePixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            PathTextureRef::OpaquePixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            PathTextureRef::TransparentPixelated,
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
        .with_entity(Texture::new(PathTextureRef::TransparentPixelated))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(ellipse(
            Vec3::new(0.15, 0.15, 3.),
            Color::rgb(1., 0., 0.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.1, 0.1, 2.),
            Color::rgb(0., 1., 0.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0., 0., 0.),
            Color::rgb(0., 0., 1.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(ellipse(
            Vec3::new(0.05, 0.05, 1.),
            Color::rgb(1., 1., 1.),
            PathTextureRef::TransparentPixelated,
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
        .with_entity(Texture::new(PathTextureRef::TransparentPixelated))
        .with_entity(Texture::new(PathTextureRef::OpaquePixelated))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .with_entity(
            rectangle(
                Vec3::new(0.15, 0.15, 3.),
                Color::rgb(1., 0., 0.),
                PathTextureRef::OpaquePixelated,
            )
            .with(AutoRemoved),
        )
        .with_entity(
            rectangle(
                Vec3::new(0.1, 0.1, 2.),
                Color::rgb(0., 1., 0.),
                PathTextureRef::TransparentPixelated,
            )
            .with(AutoRemoved),
        )
        .with_entity(
            rectangle(
                Vec3::new(0., 0., 0.),
                Color::rgb(0., 0., 1.),
                PathTextureRef::OpaquePixelated,
            )
            .with(AutoRemoved),
        )
        .with_entity(
            rectangle(
                Vec3::new(0.05, 0.05, 1.),
                Color::rgb(1., 1., 1.),
                PathTextureRef::TransparentPixelated,
            )
            .with(AutoRemoved),
        )
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_transparency_with_opaque.png")
        })
        .with_entity(rectangle(
            Vec3::new(0., 0., 1.),
            Color::rgb(1., 1., 0.),
            PathTextureRef::TransparentPixelated,
        ))
        .with_entity(rectangle(
            Vec3::new(0.25, 0.25, 2.),
            Color::rgb(1., 1., 0.),
            PathTextureRef::TransparentPixelated,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_transparency_cleaned_up.png")
        });
}
