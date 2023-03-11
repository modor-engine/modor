use crate::storages::resources::fonts::DynFontKey;
use crate::ResourceLocation;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// A trait for defining a font reference.
///
/// A font reference is generally an `enum` listing the different fonts of the application.
/// <br>This `enum` can then be used to indicate which font to load or to attach.
///
/// # Examples
///
/// See [`Font`](crate::Font).
pub trait FontRef:
    Any + Sync + Send + UnwindSafe + RefUnwindSafe + Clone + PartialEq + Eq + Hash + Debug
{
    /// Returns the associated font configuration.
    fn config(&self) -> FontConfig;
}

impl<T> DynFontKey for T where T: FontRef {}

/// The configuration of a font.
///
/// Following font formats are supported:
/// - TrueType Fonts (TTF)
/// - OpenType Fonts (OTF)
///
/// # Examples
///
/// See [`Font`](crate::Font).
#[derive(Debug)]
pub struct FontConfig {
    pub(crate) location: ResourceLocation,
}

impl FontConfig {
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
        }
    }

    /// Creates a new config from font bytes.
    ///
    /// This method can be used when the font is included directly in the code using the
    /// [`include_bytes`](macro@std::include_bytes) macro.
    pub fn from_memory(bytes: &'static [u8]) -> Self {
        Self {
            location: ResourceLocation::FromMemory(bytes),
        }
    }
}
