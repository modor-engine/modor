#[macro_use]
extern crate modor;

pub mod data;
pub mod entities;
pub mod storages;
pub mod testing;

use modor_graphics::{TextureConfig, TextureRef};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PathTextureRef {
    OpaqueSmooth,
    OpaquePixelated,
    Colored,
    TransparentPixelated,
    UnsupportedFormat,
    InvalidFormat,
    InvalidPath,
}

impl TextureRef for PathTextureRef {
    fn config(&self) -> TextureConfig {
        match self {
            Self::OpaqueSmooth => TextureConfig::from_path("../tests/assets/opaque-texture.png"),
            Self::OpaquePixelated => {
                TextureConfig::from_path("../tests/assets/opaque-texture.png").with_smooth(false)
            }
            Self::Colored => {
                TextureConfig::from_path("../tests/assets/colored-texture.png").with_smooth(false)
            }
            Self::TransparentPixelated => {
                TextureConfig::from_path("../tests/assets/transparent-texture.png")
                    .with_smooth(false)
            }
            Self::UnsupportedFormat => TextureConfig::from_path("../tests/assets/text.txt"),
            Self::InvalidFormat => {
                TextureConfig::from_path("../tests/assets/invalid-texture-format.png")
            }
            Self::InvalidPath => TextureConfig::from_path("invalid/path"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MemoryTextureRef {
    OpaqueSmooth,
    UnsupportedFormat,
    InvalidFormat,
}

impl TextureRef for MemoryTextureRef {
    fn config(&self) -> TextureConfig {
        match self {
            Self::OpaqueSmooth => {
                TextureConfig::from_memory(include_bytes!("../assets/opaque-texture.png"))
            }
            Self::UnsupportedFormat => {
                TextureConfig::from_memory(include_bytes!("../assets/text.txt"))
            }
            Self::InvalidFormat => {
                TextureConfig::from_memory(include_bytes!("../assets/invalid-texture-format.png"))
            }
        }
    }
}
