use crate::backend::textures::Image;
use crate::RenderTarget;
use image::error::UnsupportedErrorKind;
use image::{GenericImageView, ImageError};
use log::error;
use modor::{Built, EntityBuilder, SingleMut};
use modor_jobs::{AssetLoadingError, AssetLoadingJob, Job};
use std::fmt::{Debug, Display, Formatter};

// TODO: add texture label in logged errors

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
/// # use modor_graphics::{Color, Mesh2D, Texture, TextureConfig, TextureSampling};
/// # use modor_physics::Transform2D;
/// #
/// #
/// # macro_rules! include_bytes {($($path:tt)*) => { &[] }}
/// #
/// App::new()
///     .with_entity(Texture::build(TextureLabel::Rectangle))
///     .with_entity(Rectangle::build());
///
/// #[derive(Debug)]
/// enum TextureLabel {
///     Rectangle,
///     Circle,
/// }
///
/// impl From<TextureLabel> for usize {
///     fn from(label: TextureLabel) -> Self {
///         label as usize
///     }
/// }
///
/// impl From<TextureLabel> for TextureConfig {
///     fn from(label: TextureLabel) -> Self {
///         let config = match label {
///             TextureLabel::Rectangle => TextureConfig::from_path(label, "res/rectangle.png"),
///             TextureLabel::Circle => TextureConfig::from_memory(label, include_bytes!(
///                 concat!(env!("CARGO_MANIFEST_DIR"), "/assets/circle.png")
///             )),
///         };
///         config
///             .with_smaller_sampling(TextureSampling::Nearest)
///             .with_larger_sampling(TextureSampling::Linear)
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
///                 .with_texture(TextureLabel::Rectangle)
///                 .with_texture_color(Color::YELLOW)
///                 .with_color(Color::GRAY),
///         )
///     }
/// }
/// ```
pub struct Texture {
    pub(crate) id: usize,
    config: TextureConfig,
    state: TextureState,
}

#[entity]
impl Texture {
    /// Creates a new texture.
    pub fn build(config: impl Into<TextureConfig>) -> impl Built<Self> {
        let config = config.into();
        let location = config.location.clone();
        EntityBuilder::new(Self {
            id: config.texture_id,
            config,
            state: TextureState::Loading,
        })
        .with_option(if let TextureDataLocation::FromPath(p) = &location {
            Some(AssetLoadingJob::new(
                p,
                |b| async move { Self::parse_image(&b) },
            ))
        } else {
            None
        })
        .with_option(if let TextureDataLocation::FromMemory(b) = &location {
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
                    TextureState::Loaded
                }
                Ok(Some(Err(e))) => {
                    error!("cannot load texture '{}': {e}", self.config.label);
                    TextureState::Error(e)
                }
                Err(e) => {
                    error!("cannot retrieve texture '{}': {e}", self.config.label);
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
                        TextureState::Loaded
                    }
                    Err(e) => {
                        error!("cannot read texture '{}': {e}", self.config.label);
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
    pub(crate) texture_id: usize,
    pub(crate) label: String,
    pub(crate) location: TextureDataLocation,
    pub(crate) smaller_sampling: TextureSampling,
    pub(crate) larger_sampling: TextureSampling,
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
    pub fn from_path(texture_id: impl Into<usize> + Debug, path: impl Into<String>) -> Self {
        let label = format!("{texture_id:?}");
        Self {
            texture_id: texture_id.into(),
            label,
            location: TextureDataLocation::FromPath(path.into()),
            smaller_sampling: TextureSampling::Linear,
            larger_sampling: TextureSampling::Linear,
        }
    }

    /// Creates a new config from texture bytes.
    ///
    /// This method can be used when the texture is included directly in the code using the
    /// [`include_bytes`](macro@std::include_bytes) macro.
    #[must_use]
    pub fn from_memory(texture_id: impl Into<usize> + Debug, bytes: &'static [u8]) -> Self {
        let label = format!("{texture_id:?}");
        Self {
            texture_id: texture_id.into(),
            label,
            location: TextureDataLocation::FromMemory(bytes),
            smaller_sampling: TextureSampling::Linear,
            larger_sampling: TextureSampling::Linear,
        }
    }

    /// Returns the configuration with a different `smaller_sampling`.
    ///
    /// It corresponds to the sampling applied when the texture appears smaller than its
    /// original size.
    ///
    /// Default value is `TextureSampling::Linear`.
    #[must_use]
    pub fn with_smaller_sampling(mut self, smaller_sampling: TextureSampling) -> Self {
        self.smaller_sampling = smaller_sampling;
        self
    }

    /// Returns the configuration with a different `larger_sampling`.
    ///
    /// It corresponds to the sampling applied when the texture appears larger than its
    /// original size.
    ///
    /// Default value is `TextureSampling::Linear`.
    #[must_use]
    pub fn with_larger_sampling(mut self, larger_sampling: TextureSampling) -> Self {
        self.larger_sampling = larger_sampling;
        self
    }
}

/// The sampling to applied to a texture.
///
/// # Examples
///
/// See [`Texture`](crate::Texture).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureSampling {
    /// The texture appears smoothed.
    Linear,
    /// The texture appears pixelated.
    Nearest,
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
            ImageError::Limits(_) | ImageError::Parameter(_) | ImageError::IoError(_) => {
                return Err(format!("error when reading texture: {error}"))
            }
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TextureDataLocation {
    FromPath(String),
    FromMemory(&'static [u8]),
}
