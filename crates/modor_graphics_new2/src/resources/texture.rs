use crate::render_target::RenderTargetUpdate;
use crate::resources::shader::Shader;
use crate::{
    GraphicsModule, IntoResourceKey, Resource, ResourceKey, ResourceLoadingError, ResourceRegistry,
    ResourceState, Size,
};
use image::{DynamicImage, ImageBuffer, ImageError, Rgba, RgbaImage};
use modor::Single;
use modor_jobs::{AssetLoadingJob, Job};
use std::mem;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d,
    FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler, SamplerDescriptor,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub(crate) type TextureRegistry = ResourceRegistry<Texture>;

#[must_use]
#[derive(Component, Debug)]
pub struct Texture {
    key: ResourceKey,
    source: TextureSource,
    is_smooth: bool,
    state: TextureState,
}

#[systems]
impl Texture {
    pub fn from_size(key: impl IntoResourceKey, size: Size) -> Self {
        Self::new(key, TextureSource::Size(size))
    }

    pub fn from_static(key: impl IntoResourceKey, data: &'static [u8]) -> Self {
        Self::new(key, TextureSource::StaticData(data))
    }

    pub fn from_data(key: impl IntoResourceKey, data: impl Into<Vec<u8>>) -> Self {
        Self::new(key, TextureSource::Data(data.into()))
    }

    pub fn from_path(key: impl IntoResourceKey, path: impl Into<String>) -> Self {
        Self::new(key, TextureSource::Path(path.into()))
    }

    pub(crate) fn blank() -> Self {
        Self::new(TextureKey::Blank, TextureSource::Unit)
    }

    pub fn with_smooth(mut self, is_smooth: bool) -> Self {
        self.is_smooth = is_smooth;
        self
    }

    // run after RenderTarget because WindowTarget::texture_format must be run first
    #[run_after(RenderTargetUpdate)]
    fn load(&mut self, module: Option<Single<'_, GraphicsModule>>) {
        let state = mem::take(&mut self.state);
        self.state = module.map_or(TextureState::NotLoaded, |module| match state {
            TextureState::NotLoaded => self.start_loading(&module),
            TextureState::AssetLoading(job) => self.check_asset_job(job, &module),
            TextureState::DataLoading(job) => self.check_data_job(job, &module),
            TextureState::Loaded { format, .. } if format != Self::main_texture_format(&module) => {
                self.start_loading(&module)
            }
            state @ (TextureState::Loaded { .. } | TextureState::Error(_)) => state,
        });
    }

    pub(crate) fn is_transparent(&self) -> bool {
        match &self.state {
            TextureState::Loaded { is_transparent, .. } => *is_transparent,
            TextureState::NotLoaded
            | TextureState::AssetLoading(_)
            | TextureState::DataLoading(_)
            | TextureState::Error(_) => {
                panic!("internal error: texture not loaded")
            }
        }
    }

    pub(crate) fn bind_group(&self) -> &BindGroup {
        match &self.state {
            TextureState::Loaded { bind_group, .. } => bind_group,
            TextureState::NotLoaded
            | TextureState::AssetLoading(_)
            | TextureState::DataLoading(_)
            | TextureState::Error(_) => {
                panic!("internal error: texture not loaded")
            }
        }
    }

    fn new(key: impl IntoResourceKey, source: TextureSource) -> Self {
        Self {
            key: key.into_key(),
            source,
            is_smooth: true,
            state: TextureState::NotLoaded,
        }
    }

    fn start_loading(&mut self, module: &GraphicsModule) -> TextureState {
        match &self.source {
            TextureSource::Unit => {
                self.load_texture(Self::load_image_from_size(Size::new(1, 1)), module)
            }
            TextureSource::Size(size) => {
                let size = *size;
                TextureState::DataLoading(Job::new(
                    async move { Ok(Self::load_image_from_size(size)) },
                ))
            }
            TextureSource::StaticData(data) => {
                TextureState::DataLoading(Job::new(async { Self::load_image_from_memory(data) }))
            }
            TextureSource::Data(data) => {
                let data = data.clone();
                TextureState::DataLoading(Job::new(
                    async move { Self::load_image_from_memory(&data) },
                ))
            }
            TextureSource::Path(path) => {
                TextureState::AssetLoading(AssetLoadingJob::new(path, |d| async move {
                    Self::load_image_from_memory(&d)
                }))
            }
        }
    }

    fn check_asset_job(
        &mut self,
        mut job: AssetLoadingJob<Result<RgbaImage, ResourceLoadingError>>,
        module: &GraphicsModule,
    ) -> TextureState {
        match job.try_poll() {
            Ok(Some(Ok(image))) => self.load_texture(image, module),
            Ok(Some(Err(error))) => TextureState::Error(error),
            Ok(None) => TextureState::AssetLoading(job),
            Err(error) => TextureState::Error(ResourceLoadingError::AssetLoadingError(error)),
        }
    }

    fn check_data_job(
        &mut self,
        mut job: Job<Result<RgbaImage, ResourceLoadingError>>,
        module: &GraphicsModule,
    ) -> TextureState {
        match job.try_poll() {
            Ok(Some(Ok(image))) => self.load_texture(image, module),
            Ok(Some(Err(error))) => TextureState::Error(error),
            Ok(None) => TextureState::DataLoading(job),
            Err(_) => TextureState::Error(ResourceLoadingError::LoadingError(format!(
                "`{:?}` texture loading job panicked",
                self.key
            ))),
        }
    }

    fn load_texture(&mut self, image: RgbaImage, module: &GraphicsModule) -> TextureState {
        let format = Self::main_texture_format(module);
        let texture = self.create_texture(&image, format, module);
        Self::write_texture(&image, &texture, module);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = self.create_sampler(module);
        TextureState::Loaded {
            texture,
            bind_group: self.create_bind_group(&view, &sampler, module),
            format,
            is_transparent: image.pixels().any(|p| p.0[3] > 0 && p.0[3] < 255),
        }
    }

    fn create_texture(
        &self,
        image: &RgbaImage,
        format: TextureFormat,
        module: &GraphicsModule,
    ) -> wgpu::Texture {
        module.device.create_texture(&TextureDescriptor {
            label: Some(&format!("modor_texture_{:?}", self.key)),
            size: Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING // for attachment to models
                | TextureUsages::COPY_DST // for attachment to models
                | TextureUsages::RENDER_ATTACHMENT // for rendering
                | TextureUsages::COPY_SRC, // for rendering
        })
    }

    fn create_sampler(&self, module: &GraphicsModule) -> Sampler {
        module.device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("modor_texture_sampler_{:?}", self.key)),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            min_filter: FilterMode::Nearest,
            mag_filter: if self.is_smooth {
                FilterMode::Linear
            } else {
                FilterMode::Nearest
            },
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        })
    }

    fn create_bind_group(
        &self,
        view: &TextureView,
        sampler: &Sampler,
        module: &GraphicsModule,
    ) -> BindGroup {
        module.device.create_bind_group(&BindGroupDescriptor {
            layout: &module.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
            label: Some(&format!("modor_texture_bind_group_{:?}", self.key)),
        })
    }

    fn load_image_from_size(size: Size) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        RgbaImage::from_pixel(size.width, size.height, Rgba([255u8, 255, 255, 255]))
    }

    fn load_image_from_memory(data: &[u8]) -> Result<RgbaImage, ResourceLoadingError> {
        image::load_from_memory(data)
            .map_err(ResourceLoadingError::from)
            .map(DynamicImage::into_rgba8)
    }

    fn main_texture_format(module: &GraphicsModule) -> TextureFormat {
        module
            .window_texture_format
            .unwrap_or(Shader::DEFAULT_TEXTURE_FORMAT)
    }

    fn write_texture(image: &RgbaImage, texture: &wgpu::Texture, module: &GraphicsModule) {
        module.queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * image.width()),
                rows_per_image: std::num::NonZeroU32::new(image.height()),
            },
            Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
        );
    }
}

impl Resource for Texture {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        match &self.state {
            TextureState::NotLoaded => ResourceState::NotLoaded,
            TextureState::AssetLoading(_) | TextureState::DataLoading(_) => ResourceState::Loading,
            TextureState::Loaded { .. } => ResourceState::Loaded,
            TextureState::Error(error) => ResourceState::Error(error),
        }
    }
}

#[derive(Debug)]
enum TextureSource {
    Unit,
    Size(Size),
    StaticData(&'static [u8]),
    Data(Vec<u8>),
    Path(String),
}

#[derive(Debug, Default)]
enum TextureState {
    #[default]
    NotLoaded,
    AssetLoading(AssetLoadingJob<Result<RgbaImage, ResourceLoadingError>>),
    DataLoading(Job<Result<RgbaImage, ResourceLoadingError>>),
    Loaded {
        texture: wgpu::Texture,
        bind_group: BindGroup,
        format: TextureFormat,
        is_transparent: bool,
    },
    Error(ResourceLoadingError),
}

impl From<ImageError> for ResourceLoadingError {
    fn from(error: ImageError) -> Self {
        match error {
            ImageError::Decoding(e) => Self::InvalidFormat(format!("{e}")),
            ImageError::Unsupported(e) => Self::InvalidFormat(format!("{e}")),
            // coverage: off (internal errors that shouldn't happen)
            ImageError::Limits(_)
            | ImageError::Parameter(_)
            | ImageError::IoError(_)
            | ImageError::Encoding(_) => {
                Self::LoadingError(format!("error when reading texture: {error}"))
            } // coverage: on
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TextureKey {
    Blank,
}
