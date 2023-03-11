use crate::data::resources::ResourceState;
use crate::entities::render_target::RenderTarget;
use crate::storages::resources::fonts::FontKey;
use crate::{FontRef, Resource, ResourceLoading, ResourceLoadingError};
use ab_glyph::FontVec;
use modor::SingleMut;
use modor_jobs::{AssetLoadingJob, Job};

/// A font loaded asynchronously.
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
///     .with_entity(Font::new(AppFontRef::FontFromPath))
///     .with_entity(build_text());
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum AppFontRef {
///     FontFromPath,
///     FontFromMemory,
/// }
///
/// impl FontRef for AppFontRef {
///     fn config(&self) -> FontConfig {
///         match self {
///             Self::FontFromPath => FontConfig::from_path("res/font1.ttf"),
///             Self::FontFromMemory => FontConfig::from_memory(include_bytes!(
///                 concat!(env!("CARGO_MANIFEST_DIR"), "/assets/font2.otf")
///             )),
///         }
///     }
/// }
/// 
/// fn build_text() -> impl BuiltEntity {
///      EntityBuilder::new()
///          .with(Transform2D::new())
///          .with(Text2D::new(30., "Hello world!").with_font(AppFontRef::FontFromPath))
/// }
/// ```
#[derive(Component)]
pub struct Font {
    pub(crate) key: FontKey,
    state: ResourceState,
    load_from_path_job: Option<
        AssetLoadingJob<Result<<Self as ResourceLoading>::ResourceType, ResourceLoadingError>>,
    >,
    load_from_memory_job:
        Option<Job<Result<<Self as ResourceLoading>::ResourceType, ResourceLoadingError>>>,
}

#[systems]
impl Font {
    /// Creates a new font.
    pub fn new(font_ref: impl FontRef) -> Self {
        let config = font_ref.config();
        let key = FontKey::new(font_ref);
        let location = config.location;
        Self {
            key,
            state: ResourceState::Loading,
            load_from_path_job: Self::asset_loading_job(&location),
            load_from_memory_job: Self::job(&location),
        }
    }

    #[run]
    fn load(&mut self, mut target: SingleMut<'_, RenderTarget>) {
        if let Some(job) = self.load_from_path_job.as_mut() {
            <Self as ResourceLoading>::load_from_path(
                &format!("{:?}", self.key),
                &mut self.state,
                job,
                |r| target.load_font(self.key.clone(), r),
            );
        } else if let Some(job) = self.load_from_memory_job.as_mut() {
            <Self as ResourceLoading>::load_from_memory(
                &format!("{:?}", self.key),
                &mut self.state,
                job,
                |r| target.load_font(self.key.clone(), r),
            );
        }
    }
}

impl Resource for Font {
    fn state(&self) -> &ResourceState {
        &self.state
    }
}

impl ResourceLoading for Font {
    type ResourceType = FontVec;

    fn key(&self) -> String {
        format!("{:?}", self.key)
    }

    fn parse(bytes: &[u8]) -> Result<Self::ResourceType, ResourceLoadingError> {
        FontVec::try_from_vec(bytes.into())
            .map_err(|_| ResourceLoadingError::InvalidFormat("invalid font".into()))
    }
}
