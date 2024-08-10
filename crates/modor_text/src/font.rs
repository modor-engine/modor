use ab_glyph::FontVec;
use modor::{App, FromApp, Global, Globals, State, Updater};
use modor_graphics::modor_resources::{Res, ResSource, Resource, ResourceError, Source};

/// A font that can be attached to a [`Text2D`](crate::Text2D).
///
/// Following font formats are supported:
/// - TrueType Fonts (TTF)
/// - OpenType Fonts (OTF)
///
/// # Examples
///
/// See [`Text2D`](crate::Text2D).
#[derive(Debug, Global, Updater)]
pub struct Font {
    pub(crate) glob: FontGlob,
    will_change: bool,
}

impl FromApp for Font {
    fn from_app(app: &mut App) -> Self {
        app.create::<FontManager>();
        Self {
            glob: FontGlob::from_app(app),
            will_change: false,
        }
    }
}

impl Resource for Font {
    type Source = FontSource;
    type Loaded = FontVec;

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        FontVec::try_from_vec(file_bytes).map_err(|_| ResourceError::Other("invalid font".into()))
    }

    fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        match source {
            FontSource::Bytes(bytes) => FontVec::try_from_vec(bytes.to_vec())
                .map_err(|_| ResourceError::Other("invalid font".into())),
        }
    }

    fn on_load(&mut self, _app: &mut App, loaded: Self::Loaded, _source: &ResSource<Self>) {
        self.glob.font = Some(loaded);
        self.will_change = true;
    }

    fn apply_updater(_updater: Self::Updater<'_>, _app: &mut App) {
        // font has no parameter, so nothing is done
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

// TODO: merge with Font
/// The global data of a [`Font`].
#[derive(Debug, FromApp)]
pub struct FontGlob {
    pub(crate) font: Option<FontVec>,
    pub(crate) has_changed: bool,
}

// TODO: make it more direct by iterating and updating on texts when font is reloaded
#[derive(Debug, FromApp)]
struct FontManager;

impl State for FontManager {
    fn update(&mut self, app: &mut App) {
        for font in app.get_mut::<Globals<Res<Font>>>() {
            if font.will_change {
                font.will_change = false;
                font.glob.has_changed = true;
            } else {
                font.glob.has_changed = false;
            }
        }
    }
}
