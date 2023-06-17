use modor_jobs::{AssetLoadingError, AssetLoadingJob, Job};
use std::any::{self, Any};
use std::fmt::{Debug, Display, Formatter};

/// Trait for defining a loadable resource.
pub trait Resource {
    /// Returns the state of the resource.
    fn state(&self) -> &ResourceState;
}

pub(crate) trait ResourceLoading: Any + Resource {
    type ResourceType: Any + Send + Debug;

    fn parse(bytes: &[u8]) -> Result<Self::ResourceType, ResourceLoadingError>;

    fn asset_loading_job(
        location: &ResourceLocation,
    ) -> Option<AssetLoadingJob<Result<Self::ResourceType, ResourceLoadingError>>> {
        if let ResourceLocation::FromPath(p) = &location {
            Some(AssetLoadingJob::new(p, |b| async move { Self::parse(&b) }))
        } else {
            None
        }
    }

    fn job(
        location: &ResourceLocation,
    ) -> Option<Job<Result<Self::ResourceType, ResourceLoadingError>>> {
        if let ResourceLocation::FromMemory(b) = &location {
            Some(Job::new(async { Self::parse(b) }))
        } else {
            None
        }
    }

    fn load_from_path(
        key: &str,
        state: &mut ResourceState,
        job: &mut AssetLoadingJob<Result<Self::ResourceType, ResourceLoadingError>>,
        load_fn: impl FnOnce(Self::ResourceType),
    ) {
        if matches!(state, ResourceState::Loading) {
            *state = match job.try_poll() {
                Ok(Some(Ok(r))) => {
                    load_fn(r);
                    debug!(
                        "resource of type `{:?}` with reference '{:?}' loaded",
                        any::type_name::<Self>(),
                        key
                    );
                    ResourceState::Loaded
                }
                Ok(Some(Err(e))) => {
                    error!(
                        "cannot load resource of type `{:?}` with reference '{:?}': {e}",
                        any::type_name::<Self>(),
                        key
                    );
                    ResourceState::Error(e)
                }
                Err(e) => {
                    error!(
                        "cannot retrieve resource of type `{:?}` with reference '{:?}': {e}",
                        any::type_name::<Self>(),
                        key
                    );
                    ResourceState::Error(ResourceLoadingError::LoadingError(e))
                }
                Ok(None) => ResourceState::Loading,
            };
        }
    }

    fn load_from_memory(
        key: &str,
        state: &mut ResourceState,
        job: &mut Job<Result<Self::ResourceType, ResourceLoadingError>>,
        load_fn: impl FnOnce(Self::ResourceType),
    ) {
        if matches!(state, ResourceState::Loading) {
            if let Some(result) = job
                .try_poll()
                .expect("internal error: resource loading from memory has failed")
            {
                *state = match result {
                    Ok(r) => {
                        load_fn(r);
                        debug!(
                            "resource of type `{:?}` with reference '{:?}' loaded",
                            any::type_name::<Self>(),
                            key
                        );
                        ResourceState::Loaded
                    }
                    Err(e) => {
                        error!(
                            "cannot read resource of type `{:?}` with reference '{:?}': {e}",
                            any::type_name::<Self>(),
                            key
                        );
                        ResourceState::Error(e)
                    }
                };
            }
        }
    }
}

/// The state of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceState {
    /// The resource is loading.
    Loading,
    /// The resource is loaded.
    Loaded,
    /// The resource returned an error during its loading.
    Error(ResourceLoadingError),
}

/// An error that occurs during resource loading.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResourceLoadingError {
    /// The image format is unsupported.
    InvalidFormat(String),
    /// There was an error while retrieving the file.
    LoadingError(AssetLoadingError),
}

// coverage: off (not necessary to test Display impl)
#[allow(clippy::use_debug)]
impl Display for ResourceLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(e) => write!(f, "invalid format: {e:?}"),
            Self::LoadingError(e) => write!(f, "loading error: {e}"),
        }
    }
}
// coverage: on

#[derive(Debug, Clone)]
pub(crate) enum ResourceLocation {
    FromPath(String),
    FromMemory(&'static [u8]),
}
