use crate::backend::textures::Image;
use crate::data::resources::{ResourceLoadingError, ResourceLocation, ResourceState};
use crate::internal::RenderTarget;
use crate::storages::resources::textures::TextureKey;
use crate::{Resource, ResourceLoading, TextureRef};
use image::{GenericImageView, ImageError};
use modor::{Built, EntityBuilder, SingleMut};
use modor_jobs::{AssetLoadingJob, Job};

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
/// # use modor_graphics::{Color, Mesh2D, Texture, TextureRef, TextureConfig, TexturePart};
/// # use modor_physics::Transform2D;
/// # use modor_math::Vec2;
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
/// struct Rectangle;
///
/// #[entity]
/// impl Rectangle {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self).with(Transform2D::new()).with(
///             Mesh2D::rectangle()
///                 .with_texture(AppTextureRef::Rectangle)
///                 .with_texture_part(AppTexturePart::TopLeft.into())
///                 .with_texture_color(Color::YELLOW)
///                 .with_color(Color::GRAY),
///         )
///     }
/// }
/// ```
pub struct Texture {
    pub(crate) config: InternalTextureConfig,
    state: ResourceState,
}

#[entity]
impl Texture {
    /// Creates a new texture.
    pub fn build(texture_ref: impl TextureRef) -> impl Built<Self> {
        let config = texture_ref.config();
        let config = InternalTextureConfig {
            key: TextureKey::new(texture_ref),
            location: config.location,
            is_smooth: config.is_smooth,
        };
        let location = config.location.clone();
        EntityBuilder::new(Self {
            config,
            state: ResourceState::Loading,
        })
        .with_option(Self::asset_loading_job(&location))
        .with_option(Self::job(&location))
    }

    #[run]
    fn load_from_path(
        &mut self,
        job: &mut AssetLoadingJob<
            Result<<Self as ResourceLoading>::ResourceType, ResourceLoadingError>,
        >,
        target: SingleMut<'_, RenderTarget>,
    ) {
        ResourceLoading::load_from_path(self, job, target);
    }

    #[run]
    fn load_from_memory(
        &mut self,
        job: &mut Job<Result<<Self as ResourceLoading>::ResourceType, ResourceLoadingError>>,
        target: SingleMut<'_, RenderTarget>,
    ) {
        ResourceLoading::load_from_memory(self, job, target);
    }
}

impl Resource for Texture {
    fn state(&self) -> &ResourceState {
        &self.state
    }
}

impl ResourceLoading for Texture {
    type ResourceType = Image;

    fn key(&self) -> String {
        format!("{:?}", self.config.key)
    }

    fn parse(bytes: &[u8]) -> Result<Image, ResourceLoadingError> {
        image::load_from_memory(bytes)
            .map_err(|e| ResourceLoadingError::try_from(e).expect("internal error"))
            .map(|i| Image {
                is_transparent: i.pixels().any(|p| p.2 .0[3] > 0 && p.2 .0[3] < 255),
                data: i.into_rgba8(),
            })
    }

    fn load(&self, resource: Self::ResourceType, target: &mut RenderTarget) {
        target.load_texture(resource, &self.config);
    }

    fn set_state(&mut self, state: ResourceState) {
        self.state = state;
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
