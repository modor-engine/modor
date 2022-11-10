use std::iter;

use crate::backend::renderer::Renderer;
use crate::backend::textures::{Image, Texture};
use crate::InternalTextureConfig;
use fxhash::FxHashMap;
use image::{DynamicImage, ImageBuffer, Rgba};
use modor::DynKey;

pub(super) struct TextureStorage {
    default_key: DynKey,
    textures: FxHashMap<DynKey, StoredTexture>,
}

impl TextureStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        let default_texture_data = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        let default_texture_key = DynKey::new(DefaultTextureLabel);
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

    pub(super) fn default_key(&self) -> &DynKey {
        &self.default_key
    }

    pub(super) fn get_default(&self) -> &Texture {
        &self.textures[&self.default_key].texture
    }

    pub(super) fn get(&self, key: &DynKey) -> Option<&Texture> {
        self.textures.get(key).map(|t| &t.texture)
    }

    pub(super) fn is_transparent(&self, key: &DynKey) -> bool {
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
        existing_keys: impl Iterator<Item = &'a DynKey>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DefaultTextureLabel;
