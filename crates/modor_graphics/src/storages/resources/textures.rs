use super::ResourceStorage;
use crate::backend::renderer::Renderer;
use crate::backend::textures::{Image, Texture};
use crate::InternalTextureConfig;
use image::{DynamicImage, ImageBuffer, Rgba};

pub(in super::super) type TextureStorage = ResourceStorage<TextureKey, Texture>;

impl TextureStorage {
    pub(in super::super) fn new(renderer: &Renderer) -> Self {
        let default_key = TextureKey::new(DefaultTextureKey);
        let default_data = ImageBuffer::from_pixel(1, 1, Rgba([255u8, 255, 255, 255]));
        let default_resource = Texture::new(
            Image {
                data: DynamicImage::ImageRgba8(default_data).into_rgba8(),
                is_transparent: false,
            },
            false,
            false,
            "default",
            renderer,
        );
        Self::create(default_key, default_resource)
    }

    pub(in super::super) fn is_transparent(&self, texture_key: &TextureKey) -> bool {
        self.resources
            .get(texture_key)
            .map_or(false, |t| t.resource.is_transparent())
    }

    pub(in super::super) fn load(
        &mut self,
        image: Image,
        config: &InternalTextureConfig,
        is_repeated: bool,
        renderer: &Renderer,
    ) {
        self.add(
            config.key.clone(),
            Texture::new(
                image,
                config.is_smooth,
                is_repeated,
                &format!("{:?}", config.key),
                renderer,
            ),
        );
    }
}

resource_key!(TextureKey, DynTextureKey);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DefaultTextureKey;

impl DynTextureKey for DefaultTextureKey {}
