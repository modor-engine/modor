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

#[derive(Component, Debug)]
pub struct Font {
    key: ResourceKey,
    handler: ResourceHandler<LoadedFont, Vec<u8>>,
    font: Option<FontVec>,
    pub(crate) is_just_loaded: bool,
}

#[systems]
impl Font {
    pub fn new(key: impl IntoResourceKey, source: FontSource) -> Self {
        Self {
            key: key.into_key(),
            handler: ResourceHandler::new(source.into()),
            font: None,
            is_just_loaded: false,
        }
    }

    pub fn from_file(key: impl IntoResourceKey, data: &'static [u8]) -> Self {
        Self::new(key, FontSource::File(data))
    }

    pub fn from_path(key: impl IntoResourceKey, path: impl Into<String>) -> Self {
        Self::new(key, FontSource::Path(path.into()))
    }

    #[run]
    fn update(&mut self) {
        self.is_just_loaded = false;
        self.handler.update::<Self>(&self.key);
        self.font = self.font.take().or_else(|| {
            self.handler.resource().map(|r| {
                self.is_just_loaded = true;
                r.0
            })
        });
    }

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

#[non_exhaustive]
#[derive(Debug)]
pub enum FontSource {
    File(&'static [u8]),
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
