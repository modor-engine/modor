use crate::data::resources::ResourceState;
use crate::entities::render_target::RenderTarget;
use crate::storages::resources::fonts::FontKey;
use crate::{FontRef, Resource, ResourceLoading, ResourceLoadingError};
use ab_glyph::FontVec;
use modor::{Built, EntityBuilder, SingleMut};
use modor_jobs::{AssetLoadingJob, Job};

/// A font loaded asynchronously.
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
/// # use modor_graphics::{Color, Text2D, Font, FontRef, FontConfig};
/// # use modor_physics::Transform2D;
/// # use modor_math::Vec2;
/// #
/// #
/// # macro_rules! include_bytes {($($path:tt)*) => { &[] }}
/// #
/// App::new()
///     .with_entity(Font::build(AppFontRef::FontFromPath))
///     .with_entity(Text::build());
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
/// struct Text;
///
/// #[entity]
/// impl Text {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self).with(Transform2D::new()).with(
///             Text2D::new(30., "Hello world!")
///                 .with_font(AppFontRef::FontFromPath)
///         )
///     }
/// }
/// ```
pub struct Font {
    pub(crate) key: FontKey,
    state: ResourceState,
}

#[entity]
impl Font {
    /// Creates a new font.
    pub fn build(font_ref: impl FontRef) -> impl Built<Self> {
        let config = font_ref.config();
        let key = FontKey::new(font_ref);
        let location = config.location;
        EntityBuilder::new(Self {
            key,
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

    fn load(&self, resource: Self::ResourceType, target: &mut RenderTarget) {
        target.load_font(self.key.clone(), resource);
    }

    fn set_state(&mut self, state: ResourceState) {
        self.state = state;
    }
}
