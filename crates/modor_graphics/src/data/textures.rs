use super::resources::ResourceLocation;
use crate::storages::resources::textures::DynTextureKey;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// A trait for defining a texture reference.
///
/// A texture reference is generally an `enum` listing the different textures of the application.
/// <br>This `enum` can then be used to indicate which texture to load or to attach.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
pub trait TextureRef:
    Any + Sync + Send + UnwindSafe + RefUnwindSafe + Clone + PartialEq + Eq + Hash + Debug
{
    /// Returns the associated texture configuration.
    fn config(&self) -> TextureConfig;
}

impl<T> DynTextureKey for T where T: TextureRef {}

/// The configuration of a texture.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
#[derive(Debug)]
pub struct TextureConfig {
    pub(crate) location: ResourceLocation,
    pub(crate) is_smooth: bool,
}

impl TextureConfig {
    /// Creates a new config from a path relative to the asset folder.
    ///
    /// # Platform-specific
    ///
    /// - Web: HTTP GET call is performed to retrieve the file from URL
    /// `{current_browser_url}/assets/{path}`.
    /// - Android: the file is retrieved using the Android
    /// [`AssetManager`](https://developer.android.com/reference/android/content/res/AssetManager).
    /// - Other: if `CARGO_MANIFEST_DIR` environment variable is set (this is the case if the
    /// application is run using a `cargo` command), then the file is retrieved from path
    /// `{CARGO_MANIFEST_DIR}/assets/{path}`. Else, the file path is
    /// `{executable_folder_path}/assets/{path}`.
    pub fn from_path(path: impl Into<String>) -> Self {
        Self {
            location: ResourceLocation::FromPath(path.into()),
            is_smooth: true,
        }
    }

    /// Creates a new config from texture bytes.
    ///
    /// This method can be used when the texture is included directly in the code using the
    /// [`include_bytes`](macro@std::include_bytes) macro.
    pub fn from_memory(bytes: &'static [u8]) -> Self {
        Self {
            location: ResourceLocation::FromMemory(bytes),
            is_smooth: true,
        }
    }

    /// Returns the configuration with a different `is_smooth`.
    ///
    /// If `true`, a linear sampling is applied to the texture when it appears larger than its
    /// original size.
    ///
    /// Default value is `true`.
    pub fn with_smooth(mut self, is_smooth: bool) -> Self {
        self.is_smooth = is_smooth;
        self
    }
}
