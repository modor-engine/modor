use crate::resource::ResourceHandler;
use crate::{
    IntoResourceKey, Load, Resource, ResourceKey, ResourceLoadingError, ResourceSource,
    ResourceState,
};
use ab_glyph::FontVec;
use std::fmt::Debug;

#[derive(Component, Debug)]
pub struct Font {
    key: ResourceKey,
    handler: ResourceHandler<LoadedFont, Vec<u8>>,
    font: Option<FontVec>,
}

#[systems]
impl Font {
    pub fn new(key: impl IntoResourceKey, source: FontSource) -> Self {
        Self {
            key: key.into_key(),
            handler: ResourceHandler::new(source.into()),
            font: None,
        }
    }

    // TODO: notify all texts that font has been updated
    #[run]
    fn update(&mut self) {
        self.handler.update::<Self>(&self.key);
        self.font = self
            .font
            .take()
            .or_else(|| self.handler.resource().map(|r| r.0));
    }

    pub fn set_source(&mut self, source: FontSource) {
        self.handler.set_source(source.into());
    }
}

impl Resource for Font {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        self.handler.state()
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum FontSource {
    Data(Vec<u8>),
    Path(String),
}

impl From<FontSource> for ResourceSource<Vec<u8>> {
    fn from(source: FontSource) -> Self {
        match source {
            FontSource::Data(data) => Self::AsyncData(data),
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
