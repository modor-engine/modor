use crate::backend::textures::Image;
use crate::RenderTarget;
use image::error::UnsupportedErrorKind;
use image::{GenericImageView, ImageError};
use modor::{Built, EntityBuilder, SingleMut};
use modor_internal::dyn_key::DynKey;
use modor_jobs::{AssetLoadingError, AssetLoadingJob, Job};
use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

/// A texture loaded asynchronously.
///
/// # Modor
///
/// - **Type**: entity
/// - **Lifetime**: same as parent entity
/// - **Updated by**: [`GraphicsModule`](crate::GraphicsModule)
///
/// # Example
///
/// ```rust
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_graphics::{Color, Mesh2D, Texture, TextureRef, TextureConfig};
/// # use modor_physics::Transform2D;
/// #
/// #
/// # macro_rules! include_bytes {($($path:tt)*) => { &[] }}
/// #
/// App::new()
///     .with_entity(Texture::build(AppTextureRef::Rectangle))
///     .with_entity(Rectangle::build());
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum AppTextureRef {
///     Rectangle,
///     Circle,
/// }
///
/// impl TextureRef for AppTextureRef {
///     fn config(&self) -> TextureConfig {
///         let config = match self {
///             Self::Rectangle => TextureConfig::from_path("res/rectangle.png"),
///             Self::Circle => TextureConfig::from_memory(include_bytes!(
///                 concat!(env!("CARGO_MANIFEST_DIR"), "/assets/circle.png")
///             )),
///         };
///         config.with_smooth(true)
///     }
/// }
///
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self).with(Transform2D::new()).with(
///             Mesh2D::rectangle()
///                 .with_texture(AppTextureRef::Rectangle)
///                 .with_texture_color(Color::YELLOW)
///                 .with_color(Color::GRAY),
///         )
///     }
/// }
/// ```
pub struct Texture {
    pub(crate) config: InternalTextureConfig,
    state: TextureState,
}

#[entity]
impl Texture {
    /// Creates a new texture.
    pub fn build(texture_ref: impl TextureRef) -> impl Built<Self> {
        let config = texture_ref.config();
        let config = InternalTextureConfig {
            key: DynKey::new(texture_ref),
            location: config.location,
            is_smooth: config.is_smooth,
        };
        let location = config.location.clone();
        EntityBuilder::new(Self {
            config,
            state: TextureState::Loading,
        })
        .with_option(if let TextureLocation::FromPath(p) = &location {
            Some(AssetLoadingJob::new(
                p,
                |b| async move { Self::parse_image(&b) },
            ))
        } else {
            None
        })
        .with_option(if let TextureLocation::FromMemory(b) = &location {
            Some(Job::new(async { Self::parse_image(b) }))
        } else {
            None
        })
    }

    /// Returns the state of the texture.
    #[must_use]
    pub fn state(&self) -> &TextureState {
        &self.state
    }

    #[run]
    fn load_from_path(
        &mut self,
        job: &mut AssetLoadingJob<Result<Image, TextureError>>,
        mut target: SingleMut<'_, RenderTarget>,
    ) {
        if let TextureState::Loading = &self.state {
            self.state = match job.try_poll() {
                Ok(Some(Ok(i))) => {
                    target.load_texture(i, &self.config);
                    debug!("texture '{:?}' loaded", self.config.key);
                    TextureState::Loaded
                }
                Ok(Some(Err(e))) => {
                    error!("cannot load texture '{:?}': {e}", self.config.key);
                    TextureState::Error(e)
                }
                Err(e) => {
                    error!("cannot retrieve texture '{:?}': {e}", self.config.key);
                    TextureState::Error(TextureError::LoadingError(e))
                }
                Ok(None) => TextureState::Loading,
            }
        }
    }

    #[run]
    fn load_from_memory(
        &mut self,
        job: &mut Job<Result<Image, TextureError>>,
        mut target: SingleMut<'_, RenderTarget>,
    ) {
        if let TextureState::Loading = &self.state {
            if let Some(result) = job
                .try_poll()
                .expect("internal error: texture loading from memory has failed")
            {
                self.state = match result {
                    Ok(i) => {
                        target.load_texture(i, &self.config);
                        debug!("texture '{:?}' loaded", self.config.key);
                        TextureState::Loaded
                    }
                    Err(e) => {
                        error!("cannot read texture '{:?}': {e}", self.config.key);
                        TextureState::Error(e)
                    }
                }
            }
        }
    }

    fn parse_image(bytes: &[u8]) -> Result<Image, TextureError> {
        image::load_from_memory(bytes)
            .map_err(|e| TextureError::try_from(e).expect("internal error"))
            .map(|i| Image {
                is_transparent: i.pixels().any(|p| p.2 .0[3] > 0 && p.2 .0[3] < 255),
                data: i,
            })
    }
}

/// The configuration of a texture.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
#[derive(Debug)]
pub struct TextureConfig {
    pub(crate) location: TextureLocation,
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
    #[must_use]
    pub fn from_path(path: impl Into<String>) -> Self {
        Self {
            location: TextureLocation::FromPath(path.into()),
            is_smooth: true,
        }
    }

    /// Creates a new config from texture bytes.
    ///
    /// This method can be used when the texture is included directly in the code using the
    /// [`include_bytes`](macro@std::include_bytes) macro.
    #[must_use]
    pub fn from_memory(bytes: &'static [u8]) -> Self {
        Self {
            location: TextureLocation::FromMemory(bytes),
            is_smooth: true,
        }
    }

    /// Returns the configuration with a different `is_smooth`.
    ///
    /// If `true`, a linear sampling is applied to the texture when it appears larger than its
    /// original size.
    ///
    /// Default value is `true`.
    #[must_use]
    pub fn with_smooth(mut self, is_smooth: bool) -> Self {
        self.is_smooth = is_smooth;
        self
    }
}

/// The state of a texture.
#[derive(Debug, Clone, PartialEq)]
pub enum TextureState {
    /// The texture is loading.
    Loading,
    /// The texture is loaded.
    Loaded,
    /// The texture returned an error during its loading.
    Error(TextureError),
}

/// An error that occurs during texture loading.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TextureError {
    /// The image format is unsupported.
    UnsupportedFormat(UnsupportedErrorKind),
    /// The image format is invalid.
    InvalidFormat,
    /// There was an error while retrieving the file.
    LoadingError(AssetLoadingError),
}

// coverage: off (not necessary to test Display impl)
#[allow(clippy::use_debug)]
impl Display for TextureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedFormat(e) => write!(f, "unsupported image format: {e:?}"),
            Self::InvalidFormat => write!(f, "invalid image format"),
            Self::LoadingError(e) => write!(f, "loading error: {e}"),
        }
    }
}
// coverage: on

impl TryFrom<ImageError> for TextureError {
    type Error = String;

    fn try_from(error: ImageError) -> Result<Self, Self::Error> {
        Ok(match error {
            ImageError::Decoding(_) | ImageError::Encoding(_) => Self::InvalidFormat,
            ImageError::Unsupported(e) => Self::UnsupportedFormat(e.kind()),
            // coverage: off (internal errors that shouldn't happen)
            ImageError::Limits(_) | ImageError::Parameter(_) | ImageError::IoError(_) => {
                return Err(format!("error when reading texture: {error}"))
            } // coverage: on
        })
    }
}

/// A trait for defining a texture reference.
///
/// A texture reference is generally an `enum` listing the different textures of the application.
/// <br>This `enum` can then be used to indicate which texture to load or to attach.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
pub trait TextureRef:
    Any + Sync + Send + UnwindSafe + RefUnwindSafe + Clone + Eq + Hash + Debug
{
    /// Returns the associated texture configuration.
    fn config(&self) -> TextureConfig;
}

pub(crate) struct InternalTextureConfig {
    pub(crate) key: DynKey,
    pub(crate) location: TextureLocation,
    pub(crate) is_smooth: bool,
}

#[derive(Debug, Clone)]
pub(crate) enum TextureLocation {
    FromPath(String),
    FromMemory(&'static [u8]),
}
