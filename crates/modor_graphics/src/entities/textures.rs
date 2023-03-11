use crate::backend::textures::Image;
use crate::data::resources::{ResourceLoadingError, ResourceLocation, ResourceState};
use crate::entities::render_target::RenderTarget;
use crate::storages::resources::textures::TextureKey;
use crate::{Resource, ResourceLoading, TextureRef};
use image::{GenericImageView, ImageError};
use modor::SingleMut;
use modor_jobs::{AssetLoadingJob, Job};

/// A texture loaded asynchronously.
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// #
/// #
/// # macro_rules! include_bytes {($($path:tt)*) => { &[] }}
/// #
/// App::new()
///     .with_entity(Texture::new(AppTextureRef::Rectangle))
///     .with_entity(build_rectangle());
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
/// enum AppTexturePart {
///     TopLeft,
///     TopRight,
///     BottomLeft,
///     BottomRight,
/// }
///
/// impl From<AppTexturePart> for TexturePart {
///     fn from(part: AppTexturePart) -> Self {
///         let position = match part {
///             AppTexturePart::TopLeft => Vec2::new(0., 0.),
///             AppTexturePart::TopRight => Vec2::new(0.5, 0.),
///             AppTexturePart::BottomLeft => Vec2::new(0., 0.5),
///             AppTexturePart::BottomRight => Vec2::new(0.5, 0.5),
///         };
///         Self::default()
///             .with_position(position)
///             .with_size(Vec2::ONE * 0.5)
///     }
/// }
///
/// fn build_rectangle() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new())
///         .with(
///             Mesh2D::rectangle()
///                 .with_texture(AppTextureRef::Rectangle)
///                 .with_texture_part(AppTexturePart::TopLeft.into())
///                 .with_texture_color(Color::YELLOW)
///                 .with_color(Color::GRAY),
///         )
/// }
/// ```
#[derive(Component)]
pub struct Texture {
    pub(crate) config: InternalTextureConfig,
    state: ResourceState,
    load_from_path_job: Option<
        AssetLoadingJob<Result<<Self as ResourceLoading>::ResourceType, ResourceLoadingError>>,
    >,
    load_from_memory_job:
        Option<Job<Result<<Self as ResourceLoading>::ResourceType, ResourceLoadingError>>>,
}

#[systems]
impl Texture {
    /// Creates a new texture.
    pub fn new(texture_ref: impl TextureRef) -> Self {
        let config = texture_ref.config();
        let config = InternalTextureConfig {
            key: TextureKey::new(texture_ref),
            location: config.location,
            is_smooth: config.is_smooth,
        };
        let location = config.location.clone();
        Self {
            config,
            state: ResourceState::Loading,
            load_from_path_job: Self::asset_loading_job(&location),
            load_from_memory_job: Self::job(&location),
        }
    }

    #[run]
    fn load(&mut self, mut target: SingleMut<'_, RenderTarget>) {
        if let Some(job) = self.load_from_path_job.as_mut() {
            <Self as ResourceLoading>::load_from_path(
                &format!("{:?}", self.config.key),
                &mut self.state,
                job,
                |r| target.load_texture(r, &self.config),
            );
        } else if let Some(job) = self.load_from_memory_job.as_mut() {
            <Self as ResourceLoading>::load_from_memory(
                &format!("{:?}", self.config.key),
                &mut self.state,
                job,
                |r| target.load_texture(r, &self.config),
            );
        }
    }
}

impl Resource for Texture {
    fn state(&self) -> &ResourceState {
        &self.state
    }
}

impl ResourceLoading for Texture {
    type ResourceType = Image;

    fn parse(bytes: &[u8]) -> Result<Image, ResourceLoadingError> {
        image::load_from_memory(bytes)
            .map_err(|e| ResourceLoadingError::try_from(e).expect("internal error"))
            .map(|i| Image {
                is_transparent: i.pixels().any(|p| p.2 .0[3] > 0 && p.2 .0[3] < 255),
                data: i.into_rgba8(),
            })
    }
}

impl TryFrom<ImageError> for ResourceLoadingError {
    type Error = String;

    fn try_from(error: ImageError) -> Result<Self, Self::Error> {
        Ok(match error {
            ImageError::Decoding(e) => Self::InvalidFormat(format!("{e}")),
            ImageError::Unsupported(e) => Self::InvalidFormat(format!("{e}")),
            // coverage: off (internal errors that shouldn't happen)
            ImageError::Limits(_)
            | ImageError::Parameter(_)
            | ImageError::IoError(_)
            | ImageError::Encoding(_) => {
                return Err(format!("error when reading texture: {error}"))
            } // coverage: on
        })
    }
}

pub(crate) struct InternalTextureConfig {
    pub(crate) key: TextureKey,
    pub(crate) location: ResourceLocation,
    pub(crate) is_smooth: bool,
}
