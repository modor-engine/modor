use image::error::{ImageFormatHint, UnsupportedErrorKind};
use modor::{App, Built, EntityBuilder, With};
use modor_graphics::{
    testing, Capture, GraphicsModule, Mesh2D, SurfaceSize, Texture, TextureConfig, TextureError,
    TextureSampling, TextureState,
};
use modor_jobs::AssetLoadingError;
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::thread;
use std::time::Duration;

struct Rectangle;

#[entity]
impl Rectangle {
    fn build(position: Vec2, texture_id: usize) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.4, 0.3)),
            )
            .with(Mesh2D::rectangle().with_texture(texture_id))
    }
}

#[test]
fn load_textures_with_different_sampling() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(
            TextureConfig::from_path(0_usize, "../tests/assets/opaque-texture.png")
                .with_larger_sampling(TextureSampling::Nearest),
        ))
        .with_entity(Texture::build(
            TextureConfig::from_path(1_usize, "../tests/assets/opaque-texture.png")
                .with_larger_sampling(TextureSampling::Linear),
        ))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .with_entity(Rectangle::build(Vec2::new(-0.25, 0.25), 0))
        .with_entity(Rectangle::build(Vec2::new(0.25, 0.25), 1))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_sampling.png")
        });
}

#[test]
fn load_valid_texture_from_path() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(TextureConfig::from_path(
            0_usize,
            "../tests/assets/opaque-texture.png",
        )))
        .assert::<With<Texture>>(1, |e| {
            e.has(|t: &Texture| assert_eq!(t.state(), &TextureState::Loading))
        })
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .assert::<With<Texture>>(1, |e| {
            e.has(|t: &Texture| assert_eq!(t.state(), &TextureState::Loaded))
        });
}

#[test]
fn load_texture_with_unsupported_format() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(TextureConfig::from_path(
            0_usize,
            "../tests/assets/text.txt",
        )))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .assert::<With<Texture>>(1, |e| {
            e.has(|t: &Texture| {
                assert_eq!(
                    t.state(),
                    &TextureState::Error(TextureError::UnsupportedFormat(
                        UnsupportedErrorKind::Format(ImageFormatHint::Unknown)
                    ))
                );
            })
        });
}

#[test]
fn load_texture_with_invalid_format() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(TextureConfig::from_path(
            0_usize,
            "../tests/assets/invalid-texture-format.png",
        )))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .assert::<With<Texture>>(1, |e| {
            e.has(|t: &Texture| {
                assert_eq!(t.state(), &TextureState::Error(TextureError::InvalidFormat));
            })
        });
}

#[test]
fn load_texture_with_invalid_path() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::build(TextureConfig::from_path(
            0_usize,
            "invalid/path",
        )))
        .updated_until_all::<(), _>(Some(100), |t: &Texture| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), TextureState::Loading)
        })
        .assert::<With<Texture>>(1, |e| {
            e.has(|t: &Texture| {
                assert!(matches!(
                    t.state(),
                    TextureState::Error(TextureError::LoadingError(AssetLoadingError::IoError(_)))
                ));
            })
        });
}
