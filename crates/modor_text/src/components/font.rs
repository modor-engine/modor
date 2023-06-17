use ab_glyph::FontVec;
use modor_resources::{
    IntoResourceKey, Load, Resource, ResourceHandler, ResourceKey, ResourceLoadingError,
    ResourceRegistry, ResourceSource, ResourceState,
};
use std::fmt::Debug;

pub(crate) const DEFAULT_FONT_FILE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/res/Roboto-Regular.ttf"
));

pub(crate) type FontRegistry = ResourceRegistry<Font>;

/// A font that can be attached to a [`Text`](crate::Text).
///
/// Following font formats are supported:
/// - TrueType Fonts (TTF)
/// - OpenType Fonts (OTF)
///
/// # Requirements
///
/// - text [`module`](crate::module()) is initialized
///
/// # Related components
///
/// - [`Text`](crate::Text)
///
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics_new2::*;
/// # use modor_physics::*;
/// # use modor_text::*;
/// #
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with_child(Font::from_path(FontKey, "font.ttf"))
///         .with_child(text())
/// }
///
/// fn text() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Text::new("my text", 30.).with_font(FontKey))
///         .with(Texture::from_size(TextureKey, Size::ONE))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct FontKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct TextureKey;
/// ```
#[derive(Component, Debug)]
pub struct Font {
    key: ResourceKey,
    handler: ResourceHandler<LoadedFont, Vec<u8>>,
    font: Option<FontVec>,
    pub(crate) is_just_loaded: bool,
}

#[systems]
impl Font {
    /// Creates a new font identified by a unique `key` and created from `source`.
    pub fn new(key: impl IntoResourceKey, source: FontSource) -> Self {
        Self {
            key: key.into_key(),
            handler: ResourceHandler::new(source.into()),
            font: None,
            is_just_loaded: false,
        }
    }

    /// Creates a new font identified by a unique `key` and created with given file `data`.
    ///
    /// This method is equivalent to [`Font::new`] with [`FontSource::File`] source.
    pub fn from_file(key: impl IntoResourceKey, data: &'static [u8]) -> Self {
        Self::new(key, FontSource::File(data))
    }

    /// Creates a new font identified by a unique `key` and created with a given file `path`.
    ///
    /// This method is equivalent to [`Font::new`] with [`FontSource::Path`] source.
    pub fn from_path(key: impl IntoResourceKey, path: impl Into<String>) -> Self {
        Self::new(key, FontSource::Path(path.into()))
    }

    #[run]
    fn update(&mut self) {
        self.is_just_loaded = false;
        self.handler.update::<Self>(&self.key);
        self.font = if let Some(resource) = self.handler.resource() {
            self.is_just_loaded = true;
            Some(resource.0)
        } else {
            self.font.take()
        };
    }

    /// Sets the font `source` and start reloading of the font.
    ///
    /// If the previous source is already loaded, the font remains valid until the new source
    /// is loaded.
    pub fn set_source(&mut self, source: FontSource) {
        self.handler.set_source(source.into());
    }

    pub(crate) fn get(&self) -> &FontVec {
        self.font.as_ref().expect("internal error: font not loaded")
    }
}

impl Resource for Font {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.font.is_some() {
            ResourceState::Loaded
        } else {
            self.handler.state()
        }
    }
}

/// The source of a [`Font`].
///
/// Sources loaded synchronously are ready after the next [`App`](modor::App) update. Sources loaded
/// asynchronously can take more updates to be ready.
///
/// # Examples
///
/// See [`Texture`].
#[non_exhaustive]
#[derive(Debug)]
pub enum FontSource {
    /// Font loaded asynchronously from given file bytes.
    ///
    /// This variant is generally used in combination with [`include_bytes!`].
    File(&'static [u8]),
    /// Font loaded asynchronously from a given path.
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
    Path(String),
}

impl From<FontSource> for ResourceSource<Vec<u8>> {
    fn from(source: FontSource) -> Self {
        match source {
            FontSource::File(data) => Self::AsyncData(data.into()),
            FontSource::Path(path) => Self::AsyncPath(path),
        }
    }
}

#[derive(Debug)]
struct LoadedFont(FontVec);

impl Load<Vec<u8>> for LoadedFont {
    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self, ResourceLoadingError> {
        FontVec::try_from_vec(file_bytes)
            .map_err(|_| ResourceLoadingError::InvalidFormat("invalid font".into()))
            .map(Self)
    }

    fn load_from_data(data: &Vec<u8>) -> Result<Self, ResourceLoadingError> {
        Self::load_from_file(data.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum FontKey {
    Default,
}
