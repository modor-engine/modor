use std::{fmt, iter};

use crate::backend::renderer::Renderer;
use crate::backend::textures::{Image, Texture};
use crate::{DynTextureKey, InternalTextureConfig, TextureConfig, TextureRef};
use fxhash::FxHashMap;
use image::{DynamicImage, ImageBuffer, Rgba};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

pub(super) struct TextureStorage {
    default_key: TextureKey,
    textures: FxHashMap<TextureKey, StoredTexture>,
}

impl TextureStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        let default_texture_data = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        let default_texture_key = TextureKey::new(DefaultTextureKey);
        Self {
            textures: iter::once((
                default_texture_key.clone(),
                StoredTexture {
                    texture: Texture::new(
                        Image {
                            data: DynamicImage::ImageRgba8(default_texture_data),
                            is_transparent: false,
                        },
                        false,
                        "default",
                        renderer,
                    ),
                    is_deleted: false,
                },
            ))
            .collect(),
            default_key: default_texture_key,
        }
    }

    pub(super) fn default_key(&self) -> &TextureKey {
        &self.default_key
    }

    pub(super) fn get_default(&self) -> &Texture {
        &self.textures[&self.default_key].texture
    }

    pub(super) fn get(&self, key: &TextureKey) -> Option<&Texture> {
        self.textures.get(key).map(|t| &t.texture)
    }

    pub(super) fn is_transparent(&self, key: &TextureKey) -> bool {
        self.get(key).map_or(false, Texture::is_transparent)
    }

    pub(super) fn load_texture(
        &mut self,
        image: Image,
        config: &InternalTextureConfig,
        renderer: &Renderer,
    ) {
        self.textures.insert(
            config.key.clone(),
            StoredTexture {
                texture: Texture::new(
                    image,
                    config.is_smooth,
                    &format!("{:?}", config.key),
                    renderer,
                ),
                is_deleted: false,
            },
        );
    }

    pub(crate) fn remove_not_found_textures<'a>(
        &mut self,
        existing_keys: impl Iterator<Item = &'a TextureKey>,
    ) {
        for (key, texture) in &mut self.textures {
            if key != &self.default_key {
                texture.is_deleted = true;
            }
        }
        for key in existing_keys {
            if let Some(texture) = self.textures.get_mut(key) {
                texture.is_deleted = false;
            }
        }
        self.textures.retain(|_, t| !t.is_deleted);
    }
}

struct StoredTexture {
    texture: Texture,
    is_deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DefaultTextureKey;

impl TextureRef for DefaultTextureKey {
    // coverage: off (unreachable)
    fn config(&self) -> TextureConfig {
        unreachable!("internal error: unreachable config generation from `DefaultTextureKey`")
    }
    // coverage: on
}

pub(crate) struct TextureKey(Box<dyn DynTextureKey>);

impl TextureKey {
    pub(crate) fn new(texture_ref: impl DynTextureKey) -> Self {
        Self(Box::new(texture_ref))
    }
}

impl Clone for TextureKey {
    fn clone(&self) -> Self {
        Self(self.0.as_ref().dyn_clone())
    }
}

impl PartialEq for TextureKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.dyn_partial_eq(other.0.as_dyn_partial_eq())
    }
}

impl Eq for TextureKey {}

impl Hash for TextureKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.dyn_hash(state);
    }
}

impl Debug for TextureKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.dyn_fmt(f)
    }
}

dyn_clone_trait!(pub DynTextureKeyClone, DynTextureKey);
