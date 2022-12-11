use super::resources::fonts::{FontKey, FontStorage};
use super::resources::textures::{DynTextureKey, TextureKey, TextureStorage};
use crate::backend::renderer::Renderer;
use crate::backend::textures::Image;
use crate::utils::texts;
use crate::{InternalTextureConfig, ResourceLocation, Text2D};
use ab_glyph::FontVec;
use fxhash::FxHashSet;
use image::RgbaImage;
use modor_internal::ti_vec::TiVecSafeOperations;
use modor_math::Vec2;
use typed_index_collections::TiVec;

idx_type!(pub(crate) TextIdx);

#[derive(Default)]
pub(super) struct TextStorage {
    properties: TiVec<TextIdx, Option<TextProperties>>,
    available_idxs: Vec<TextIdx>,
    logged_missing_font_keys: FxHashSet<FontKey>,
}

impl TextStorage {
    pub(super) fn reset(&mut self) {
        for properties in self.properties.iter_mut().flatten() {
            properties.is_registered = false;
        }
    }

    pub(super) fn texture_keys(&self) -> impl Iterator<Item = &TextureKey> {
        self.properties.iter().flatten().map(|p| &p.texture_key)
    }

    pub(super) fn register(
        &mut self,
        text: &mut Text2D,
        fonts: &FontStorage,
        textures: &mut TextureStorage,
        renderer: &mut Renderer,
    ) -> (TextureKey, Vec2) {
        if let Some(text_idx) = text.text_idx {
            if let Some(Some(properties)) = self.properties.get_mut(text_idx) {
                if !properties.is_registered {
                    if Self::text_has_changed(properties, text, fonts) {
                        *properties = Self::create_text(
                            text_idx,
                            text,
                            fonts,
                            textures,
                            renderer,
                            &mut self.logged_missing_font_keys,
                        );
                    } else {
                        properties.is_registered = true;
                    }
                    return (properties.texture_key.clone(), properties.texture_size);
                }
            }
            text.text_idx = None;
        }
        let text_idx = self
            .available_idxs
            .pop()
            .unwrap_or_else(|| self.properties.next_key());
        let properties = Self::create_text(
            text_idx,
            text,
            fonts,
            textures,
            renderer,
            &mut self.logged_missing_font_keys,
        );
        let texture_key = properties.texture_key.clone();
        let texture_size = properties.texture_size;
        *self.properties.get_mut_or_create(text_idx) = Some(properties);
        (texture_key, texture_size)
    }

    pub(super) fn delete_unregistered(&mut self) {
        for text_idx in self.properties.keys() {
            let properties = &mut self.properties[text_idx];
            if let Some(TextProperties {
                is_registered: false,
                ..
            }) = properties
            {
                *properties = None;
                self.available_idxs.push(text_idx);
            }
        }
    }

    fn text_has_changed(properties: &TextProperties, text: &Text2D, fonts: &FontStorage) -> bool {
        properties.string != text.string
            || &properties.font_key != Self::text_font(text, fonts, None).0
            || (properties.font_height - text.font_height).abs() > f32::EPSILON
    }

    #[allow(clippy::cast_precision_loss)]
    fn create_text(
        text_idx: TextIdx,
        text: &mut Text2D,
        fonts: &FontStorage,
        textures: &mut TextureStorage,
        renderer: &mut Renderer,
        logged_missing_font_keys: &mut FxHashSet<FontKey>,
    ) -> TextProperties {
        let (font_key, font) = Self::text_font(text, fonts, Some(logged_missing_font_keys));
        let texture_key = TextureKey::new(TextTextureKey(text_idx));
        let mut texture =
            texts::generate_texture(&text.string, text.alignment, text.font_height, font);
        let texture_size = Vec2::new(texture.dimensions().0 as f32, texture.dimensions().1 as f32);
        Self::remove_outline(&mut texture);
        textures.load(
            Image {
                data: texture,
                is_transparent: true,
            },
            &InternalTextureConfig {
                key: texture_key.clone(),
                location: ResourceLocation::FromMemory(&[]),
                is_smooth: true,
            },
            false,
            renderer,
        );
        let font_key = font_key.clone();
        text.text_idx = Some(text_idx);
        TextProperties {
            texture_key,
            texture_size,
            is_registered: true,
            string: text.string.clone(),
            font_key,
            font_height: text.font_height,
        }
    }

    fn text_font<'a>(
        text: &'a Text2D,
        fonts: &'a FontStorage,
        logged_missing_font_keys: Option<&mut FxHashSet<FontKey>>,
    ) -> (&'a FontKey, &'a FontVec) {
        let font_key = &text.font_key;
        fonts.get(font_key).map_or_else(
            || {
                if let Some(logged_missing_font_keys) = logged_missing_font_keys {
                    if !logged_missing_font_keys.contains(font_key) {
                        error!("font with ID '{:?}' attached but not loaded", font_key);
                        logged_missing_font_keys.insert(font_key.clone());
                    }
                }
                (fonts.default_key(), fonts.get_default())
            },
            |font| (&text.font_key, font),
        )
    }

    // used to avoid display artifacts
    fn remove_outline(image: &mut RgbaImage) {
        let (width, height) = image.dimensions();
        for x in 0..width {
            image.get_pixel_mut(x, 0).0[3] = 0;
            image.get_pixel_mut(x, height - 1).0[3] = 0;
        }
        for y in 0..height {
            image.get_pixel_mut(0, y).0[3] = 0;
            image.get_pixel_mut(width - 1, y).0[3] = 0;
        }
    }
}

struct TextProperties {
    texture_key: TextureKey,
    texture_size: Vec2,
    is_registered: bool,
    string: String,
    font_key: FontKey,
    font_height: f32,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
struct TextTextureKey(TextIdx);

impl DynTextureKey for TextTextureKey {}
