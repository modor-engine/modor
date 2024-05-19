use crate::gpu::{Gpu, GpuManager};
use crate::size::NonZeroSize;
use crate::texture::internal::TextureLoaded;
use crate::Size;
use image::{DynamicImage, RgbaImage};
use modor::{Context, Glob, GlobRef};
use modor_resources::{Resource, ResourceError, Source};
use wgpu::{
    AddressMode, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler,
    SamplerDescriptor, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};

// TODO: add buffer access + render target

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Texture {
    /// Whether the texture is smooth.
    ///
    /// If `true`, a linear sampling is applied to the texture when it is rendered larger than its
    /// original size.
    ///
    /// Default is `true`.
    pub is_smooth: bool,
    /// Whether the texture is repeated.
    ///
    /// If `true`, the texture is rendered repeated when the texture width or height configured in
    /// an associated [`Material`](crate::Material) is greater than `1.0`.
    ///
    /// Default is `false`.
    pub is_repeated: bool,
    loaded: TextureLoaded,
    glob: Glob<TextureGlob>,
    old_is_smooth: bool,
    old_is_repeated: bool,
}

impl Resource for Texture {
    type Source = TextureSource;
    type Loaded = TextureLoaded;

    fn create(ctx: &mut Context<'_>) -> Self {
        let loaded = TextureLoaded::default();
        let glob = TextureGlob::new(
            ctx,
            &loaded,
            Self::DEFAULT_IS_REPEATED,
            Self::DEFAULT_IS_SMOOTH,
            "default(modor_graphics)",
        );
        Self {
            is_smooth: Self::DEFAULT_IS_SMOOTH,
            is_repeated: Self::DEFAULT_IS_REPEATED,
            loaded,
            glob: Glob::new(ctx, glob),
            old_is_smooth: Self::DEFAULT_IS_SMOOTH,
            old_is_repeated: Self::DEFAULT_IS_REPEATED,
        }
    }

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        Self::load_from_file(&file_bytes).map(Into::into)
    }

    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        Ok(TextureLoaded::from(match source {
            TextureSource::Size(size) => Self::load_from_size(*size, None)?,
            TextureSource::Buffer(size, buffer) => Self::load_from_size(*size, Some(buffer))?,
            TextureSource::Bytes(bytes) => Self::load_from_file(bytes)?,
        }))
    }

    fn update(&mut self, ctx: &mut Context<'_>, loaded: Option<Self::Loaded>, label: &str) {
        if let Some(loaded) = loaded {
            self.loaded = loaded;
            self.init_glob(ctx, label);
        } else if self.old_is_smooth != self.is_smooth || self.old_is_repeated != self.is_repeated {
            self.update_glob(ctx, label);
            self.old_is_smooth = self.is_smooth;
            self.old_is_repeated = self.is_repeated;
        }
    }
}

impl Texture {
    pub(crate) const DEFAULT_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub(crate) const DEFAULT_IS_SMOOTH: bool = true;
    pub(crate) const DEFAULT_IS_REPEATED: bool = false;

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<TextureGlob> {
        self.glob.as_ref()
    }

    fn load_from_file(data: &[u8]) -> Result<RgbaImage, ResourceError> {
        image::load_from_memory(data)
            .map_err(|err| ResourceError::Other(format!("{err}")))
            .map(DynamicImage::into_rgba8)
    }

    fn load_from_size(size: Size, buffer: Option<&[u8]>) -> Result<RgbaImage, ResourceError> {
        let size = Size::from(NonZeroSize::from(size)); // ensure resolution of at least 1x1
        let buffer = if let Some(buffer) = buffer {
            buffer.to_vec()
        } else {
            vec![255; (size.width * size.height * 4) as usize] // faster than RgbaImage::from_pixel
        };
        let buffer_len = buffer.len();
        RgbaImage::from_raw(size.width, size.height, buffer).ok_or_else(|| {
            ResourceError::Other(format!(
                "RGBA buffer ({buffer_len} bytes) does not correspond to image size ({size:?})",
            ))
        })
    }

    fn init_glob(&mut self, ctx: &mut Context<'_>, label: &str) {
        *self.glob.get_mut(ctx) =
            TextureGlob::new(ctx, &self.loaded, self.is_repeated, self.is_smooth, label);
    }

    fn update_glob(&mut self, ctx: &mut Context<'_>, label: &str) {
        let gpu = ctx.get_mut::<GpuManager>().get().clone();
        self.glob
            .get_mut(ctx)
            .update_sampler(&gpu, self.is_repeated, self.is_smooth, label);
    }
}

/// The source of a [`Res`](modor_resources::Res)`<`[`Texture`]`>`.
///
/// # Examples
///
/// See [`Texture`].
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TextureSource {
    /// White texture created synchronously with a given size.
    ///
    /// If width or height is zero, then the created texture will have a size of 1x1.
    Size(Size),
    /// Texture loaded synchronously from given size and RGBA buffer.
    ///
    /// If width or height is zero, then a white texture is created with size 1x1.
    Buffer(Size, Vec<u8>),
    /// Texture loaded asynchronously from bytes.
    ///
    /// This variant is generally used in combination with [`include_bytes!`].
    Bytes(&'static [u8]),
}

impl Source for TextureSource {
    fn is_async(&self) -> bool {
        match self {
            Self::Size(_) | Self::Buffer(_, _) => false,
            Self::Bytes(_) => true,
        }
    }
}

pub struct TextureGlob {
    pub size: Size,
    pub(crate) is_transparent: bool,
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    texture: wgpu::Texture,
}

impl TextureGlob {
    fn new(
        ctx: &mut Context<'_>,
        loaded: &TextureLoaded,
        is_repeated: bool,
        is_smooth: bool,
        label: &str,
    ) -> Self {
        let gpu = ctx.get_mut::<GpuManager>().get();
        let texture = Self::create_texture(gpu, loaded, label);
        Self::write_texture(gpu, loaded, &texture);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = Self::create_sampler(gpu, is_repeated, is_smooth, label);
        Self {
            size: Size::new(loaded.image.width(), loaded.image.height()),
            is_transparent: loaded.is_transparent,
            texture,
            view,
            sampler,
        }
    }

    fn update_sampler(&mut self, gpu: &Gpu, is_repeated: bool, is_smooth: bool, label: &str) {
        self.sampler = Self::create_sampler(gpu, is_repeated, is_smooth, label);
    }

    fn create_texture(gpu: &Gpu, loaded: &TextureLoaded, label: &str) -> wgpu::Texture {
        gpu.device.create_texture(&TextureDescriptor {
            label: Some(&format!("modor_texture:{label}")),
            size: Extent3d {
                width: loaded.image.width(),
                height: loaded.image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Texture::DEFAULT_FORMAT,
            usage: TextureUsages::TEXTURE_BINDING // to use the texture
                | TextureUsages::COPY_DST // to use the texture
                | TextureUsages::RENDER_ATTACHMENT // to render in the texture
                | TextureUsages::COPY_SRC, // to render in the texture
            view_formats: &[],
        })
    }

    fn write_texture(context: &Gpu, loaded: &TextureLoaded, texture: &wgpu::Texture) {
        context.queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &loaded.image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * loaded.image.width()),
                rows_per_image: Some(loaded.image.height()),
            },
            Extent3d {
                width: loaded.image.width(),
                height: loaded.image.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    fn create_sampler(gpu: &Gpu, is_repeated: bool, is_smooth: bool, label: &str) -> Sampler {
        let address_mode = if is_repeated {
            AddressMode::Repeat
        } else {
            AddressMode::ClampToEdge
        };
        gpu.device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("modor_texture_sampler:{label}")),
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            min_filter: FilterMode::Nearest,
            mag_filter: if is_smooth {
                FilterMode::Linear
            } else {
                FilterMode::Nearest
            },
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        })
    }
}

mod internal {
    use image::{Rgba, RgbaImage};

    #[derive(Debug)]
    pub struct TextureLoaded {
        pub(super) image: RgbaImage,
        pub(super) is_transparent: bool,
    }

    impl Default for TextureLoaded {
        fn default() -> Self {
            Self {
                image: RgbaImage::from_pixel(1, 1, Rgba::<u8>::from([255, 255, 255, 255])),
                is_transparent: false,
            }
        }
    }

    impl From<RgbaImage> for TextureLoaded {
        fn from(image: RgbaImage) -> Self {
            Self {
                is_transparent: image.pixels().any(|p| p.0[3] > 0 && p.0[3] < 255),
                image,
            }
        }
    }
}
