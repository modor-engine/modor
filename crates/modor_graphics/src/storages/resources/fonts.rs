use super::ResourceStorage;
use ab_glyph::FontVec;

const DEFAULT_FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/res/Roboto-Regular.ttf"
));

pub(in super::super) type FontStorage = ResourceStorage<FontKey, FontVec>;

impl FontStorage {
    pub(in super::super) fn new() -> Self {
        let default_key = FontKey::new(DefaultFontKey);
        let default_resource = FontVec::try_from_vec(DEFAULT_FONT_DATA.into())
            .expect("internal error: cannot load default font");
        Self::create(default_key, default_resource)
    }

    pub(in super::super) fn load(&mut self, key: FontKey, font: FontVec) {
        self.add(key, font);
    }
}

resource_key!(FontKey, DynFontKey);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DefaultFontKey;

impl DynFontKey for DefaultFontKey {}
