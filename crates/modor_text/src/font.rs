use ab_glyph::FontVec;
use modor::{App, Glob, GlobRef};
use modor_graphics::modor_resources::{Resource, ResourceError, Source};

/// A font that can be attached to a [`Text2D`](crate::Text2D).
///
/// Following font formats are supported:
/// - TrueType Fonts (TTF)
/// - OpenType Fonts (OTF)
///
/// # Examples
///
/// See [`Text2D`](crate::Text2D).
pub struct Font {
    label: String,
    glob: Glob<FontGlob>,
}

impl Resource for Font {
    type Source = FontSource;
    type Loaded = FontVec;

    fn label(&self) -> &str {
        &self.label
    }

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        FontVec::try_from_vec(file_bytes).map_err(|_| ResourceError::Other("invalid font".into()))
    }

    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        match source {
            FontSource::Bytes(bytes) => FontVec::try_from_vec(bytes.to_vec())
                .map_err(|_| ResourceError::Other("invalid font".into())),
        }
    }

    fn update(&mut self, app: &mut App, loaded: Option<Self::Loaded>) {
        let glob = self.glob.get_mut(app);
        if let Some(loaded) = loaded {
            glob.font = Some(loaded);
            glob.has_changed = true;
        } else {
            glob.has_changed = false;
        }
    }
}

impl Font {
    /// Creates a new font.
    ///
    /// The `label` is used to identity the font in logs.
    pub fn new(app: &mut App, label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            glob: Glob::new(app, FontGlob::default()),
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<FontGlob> {
        self.glob.as_ref()
    }
}

/// The source of a [`Font`].
///
/// # Examples
///
/// See [`Font`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum FontSource {
    /// Font loaded asynchronously from bytes.
    ///
    /// This variant is generally used in combination with [`include_bytes!`].
    Bytes(&'static [u8]),
}

impl Source for FontSource {
    fn is_async(&self) -> bool {
        match self {
            Self::Bytes(_) => true,
        }
    }
}

/// The global data of a [`Font`].
#[derive(Debug, Default)]
pub struct FontGlob {
    pub(crate) font: Option<FontVec>,
    pub(crate) has_changed: bool,
}
