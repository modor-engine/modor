use crate::backend::renderer::Renderer;
use crate::backend::textures::{Image, Texture};
use crate::{TextureConfig, TextureSampling};
use image::{DynamicImage, ImageBuffer, Rgba};
use modor_internal::ti_vec::TiVecSafeOperations;
use typed_index_collections::TiVec;

pub(super) struct TextureStorage {
    textures: TiVec<TextureIdx, Option<StoredTexture>>,
}

impl TextureStorage {
    pub(super) const DEFAULT_TEXTURE_IDX: TextureIdx = TextureIdx(0);

    pub(super) fn new(renderer: &Renderer) -> Self {
        let default_texture_data = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        Self {
            textures: ti_vec![Some(StoredTexture {
                texture: Texture::new(
                    Image {
                        data: DynamicImage::ImageRgba8(default_texture_data),
                        is_transparent: false
                    },
                    false,
                    false,
                    &Self::DEFAULT_TEXTURE_IDX.0.to_string(),
                    renderer,
                ),
                should_be_removed: false
            })],
        }
    }

    pub(super) fn get_default(&self) -> &Texture {
        &self.textures[Self::DEFAULT_TEXTURE_IDX]
            .as_ref()
            .expect("internal error: default texture not loaded")
            .texture
    }

    pub(super) fn get(&self, idx: TextureIdx) -> Option<&Texture> {
        self.textures
            .get(idx)
            .and_then(Option::as_ref)
            .map(|t| &t.texture)
    }

    pub(super) fn is_transparent(&self, idx: TextureIdx) -> bool {
        self.get(idx).map_or(false, Texture::is_transparent)
    }

    pub(super) fn load_texture(
        &mut self,
        image: Image,
        config: &TextureConfig,
        renderer: &Renderer,
    ) {
        let id = config.texture_id + 1;
        *self.textures.get_mut_or_create(id.into()) = Some(StoredTexture {
            texture: Texture::new(
                image,
                matches!(config.smaller_sampling, TextureSampling::Linear),
                matches!(config.larger_sampling, TextureSampling::Linear),
                &id.to_string(),
                renderer,
            ),
            should_be_removed: false,
        });
    }

    pub(crate) fn remove_not_found_textures(
        &mut self,
        texture_idxs: impl Iterator<Item = TextureIdx>,
    ) {
        for texture in self.textures.iter_mut().skip(1).flatten() {
            texture.should_be_removed = true;
        }
        for texture_idx in texture_idxs {
            if let Some(Some(texture)) = self.textures.get_mut(texture_idx) {
                texture.should_be_removed = false;
            }
        }
        for texture in &mut self.textures {
            if let Some(StoredTexture {
                should_be_removed: true,
                ..
            }) = texture
            {
                texture.take();
            }
        }
    }
}

struct StoredTexture {
    texture: Texture,
    should_be_removed: bool,
}

idx_type!(pub(super) TextureIdx);
