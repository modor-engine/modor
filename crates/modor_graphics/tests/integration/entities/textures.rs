use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::{
    testing, Capture, GraphicsModule, Mesh2D, Resource, ResourceLoadingError, ResourceState,
    SurfaceSize, Texture, TextureRef,
};
use modor_jobs::AssetLoadingError;
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::thread;
use std::time::Duration;

use crate::{MemoryTextureRef, PathTextureRef};

#[derive(Component)]
struct TextureState(ResourceState);

#[systems]
impl TextureState {
    #[run_after(component(Texture))]
    fn update(&mut self, texture: &Texture) {
        self.0 = texture.state().clone();
    }
}

fn rectangle(position: Vec2, texture_ref: impl TextureRef) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.4, 0.3)),
        )
        .with(Mesh2D::rectangle().with_texture(texture_ref))
}

fn texture(texture_ref: impl TextureRef) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::new(texture_ref))
        .with(TextureState(ResourceState::Loading))
}

#[test]
fn load_textures_with_different_sampling() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Texture::new(PathTextureRef::OpaquePixelated))
        .with_entity(Texture::new(PathTextureRef::OpaqueSmooth))
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .with_entity(rectangle(
            Vec2::new(-0.25, 0.25),
            PathTextureRef::OpaquePixelated,
        ))
        .with_entity(rectangle(
            Vec2::new(0.25, 0.25),
            PathTextureRef::OpaqueSmooth,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/texture_sampling.png")
        });
}

#[test]
fn load_valid_texture_from_path() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(PathTextureRef::OpaqueSmooth))
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| assert_eq!(s.0, ResourceState::Loading))
        })
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| assert_eq!(s.0, ResourceState::Loaded))
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| assert_eq!(s.0, ResourceState::Loaded))
        });
}

#[test]
fn load_texture_from_path_with_unsupported_format() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(PathTextureRef::UnsupportedFormat))
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        });
}

#[test]
fn load_texture_from_path_with_invalid_format() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(PathTextureRef::InvalidFormat))
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        });
}

#[test]
fn load_texture_from_path_with_invalid_path() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(PathTextureRef::InvalidPath))
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::LoadingError(
                        AssetLoadingError::IoError(_)
                    ))
                ));
            })
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::LoadingError(
                        AssetLoadingError::IoError(_)
                    ))
                ));
            })
        });
}

#[test]
fn load_valid_texture_from_memory() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(MemoryTextureRef::OpaqueSmooth))
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| assert_eq!(s.0, ResourceState::Loading))
        })
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| assert_eq!(s.0, ResourceState::Loaded))
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| assert_eq!(s.0, ResourceState::Loaded))
        });
}

#[test]
fn load_texture_from_memory_with_unsupported_format() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(MemoryTextureRef::UnsupportedFormat))
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        });
}

#[test]
fn load_texture_from_memory_with_invalid_format() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(texture(MemoryTextureRef::InvalidFormat))
        .updated_until_all::<(), _>(Some(100), |s: &TextureState| {
            thread::sleep(Duration::from_millis(10));
            !matches!(s.0, ResourceState::Loading)
        })
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        })
        .updated()
        .assert::<With<TextureState>>(1, |e| {
            e.has(|s: &TextureState| {
                assert!(matches!(
                    s.0,
                    ResourceState::Error(ResourceLoadingError::InvalidFormat(_))
                ));
            })
        });
}
