use crate::gpu::{Gpu, GpuHandle, GpuResourceAction};
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

#[derive(Debug)]
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
    loaded: Option<TextureLoaded>,
    glob: Glob<Option<TextureGlob>>,
    gpu: GpuHandle,
}

impl Resource for Texture {
    type Source = TextureSource;
    type Loaded = TextureLoaded;

    fn create(ctx: &mut Context<'_>) -> Self {
        Self {
            is_smooth: true,
            is_repeated: false,
            loaded: None,
            glob: Glob::new(ctx, None),
            gpu: GpuHandle::default(),
        }
    }

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        Self::load_from_file(&file_bytes).map(|image| TextureLoaded { image })
    }

    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        Ok(TextureLoaded {
            image: match source {
                TextureSource::Size(size) => Self::load_from_size(*size, None)?,
                TextureSource::Buffer(size, buffer) => Self::load_from_size(*size, Some(buffer))?,
                TextureSource::Bytes(bytes) => Self::load_from_file(bytes)?,
            },
        })
    }

    fn update(&mut self, ctx: &mut Context<'_>, loaded: Option<Self::Loaded>, label: &str) {
        let is_loaded = loaded.is_some();
        if let Some(loaded) = loaded {
            self.loaded = Some(loaded);
        }
        match self.gpu.action(ctx, is_loaded) {
            GpuResourceAction::Delete => *self.glob.get_mut(ctx) = None,
            GpuResourceAction::Create(gpu) => self.create_glob(ctx, &gpu, label),
            GpuResourceAction::Update(gpu) => self.update_glob(ctx, &gpu, label),
        }
    }
}

impl Texture {
    pub(crate) const DEFAULT_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Option<TextureGlob>> {
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
            vec![255; (size.width * size.height * 4) as usize]
        };
        let buffer_len = buffer.len();
        RgbaImage::from_raw(size.width, size.height, buffer).ok_or_else(|| {
            ResourceError::Other(format!(
                "RGBA buffer ({buffer_len} bytes) does not correspond to image size ({size:?})",
            ))
        })
    }

    fn create_glob(&self, ctx: &mut Context<'_>, gpu: &Gpu, label: &str) {
        if let Some(loaded) = &self.loaded {
            *self.glob.get_mut(ctx) = Some(TextureGlob::new(
                gpu,
                loaded,
                self.is_repeated,
                self.is_smooth,
                label,
            ));
        }
    }

    fn update_glob(&self, ctx: &mut Context<'_>, gpu: &Gpu, label: &str) {
        if let Some(glob) = self.glob.get_mut(ctx) {
            glob.update(gpu, self.is_repeated, self.is_smooth, label);
        }
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
    is_transparent: bool,
    texture: wgpu::Texture,
    view: TextureView,
    sampler: Sampler,
    is_repeated: bool,
    is_smooth: bool,
    gpu_version: u64,
}

impl TextureGlob {
    fn new(
        gpu: &Gpu,
        loaded: &TextureLoaded,
        is_repeated: bool,
        is_smooth: bool,
        label: &str,
    ) -> Self {
        let texture = Self::create_texture(gpu, loaded, label);
        Self::write_texture(gpu, loaded, &texture);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = Self::create_sampler(gpu, is_repeated, is_smooth, label);
        Self {
            size: Size::new(loaded.image.width(), loaded.image.height()),
            is_transparent: loaded.is_transparent(),
            texture,
            view,
            sampler,
            is_repeated,
            is_smooth,
            gpu_version: gpu.version,
        }
    }

    pub(crate) fn properties(&self, gpu_version: u64) -> Option<TextureProperties<'_>> {
        (self.gpu_version == gpu_version).then_some(TextureProperties {
            is_transparent: self.is_transparent,
            view: &self.view,
            sampler: &self.sampler,
        })
    }

    fn update(&mut self, gpu: &Gpu, is_repeated: bool, is_smooth: bool, label: &str) {
        if self.is_repeated != is_repeated || self.is_smooth != is_smooth {
            self.sampler = Self::create_sampler(gpu, is_repeated, is_smooth, label);
            self.is_repeated = is_repeated;
            self.is_smooth = is_smooth;
        }
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextureProperties<'a> {
    pub(crate) is_transparent: bool,
    pub(crate) view: &'a TextureView,
    pub(crate) sampler: &'a Sampler,
}

mod internal {
    use image::RgbaImage;

    #[derive(Debug)]
    pub struct TextureLoaded {
        pub(super) image: RgbaImage,
    }

    impl TextureLoaded {
        pub(super) fn is_transparent(&self) -> bool {
            self.image.pixels().any(|p| p.0[3] > 0 && p.0[3] < 255)
        }
    }
}
