use crate::components::renderer::Renderer;
use crate::components::shader::Shader;
use crate::data::size::NonZeroSize;
use crate::{
    GpuContext, IntoResourceKey, Load, Resource, ResourceHandler, ResourceKey,
    ResourceLoadingError, ResourceRegistry, ResourceSource, ResourceState, Size,
};
use image::{DynamicImage, ImageError, Rgba, RgbaImage};
use modor::Single;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d,
    FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler, SamplerDescriptor,
    TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub(crate) type TextureRegistry = ResourceRegistry<Texture>;

#[must_use]
#[derive(Component, Debug)]
pub struct Texture {
    key: ResourceKey,
    is_smooth: bool,
    is_repeated: bool,
    handler: ResourceHandler<LoadedImage, TextureData>,
    texture: Option<LoadedTexture>,
    renderer_version: Option<u8>,
}

#[systems]
impl Texture {
    pub fn new(key: impl IntoResourceKey, source: TextureSource) -> Self {
        Self {
            key: key.into_key(),
            is_smooth: true,
            is_repeated: false,
            handler: ResourceHandler::new(source.into()),
            texture: None,
            renderer_version: None,
        }
    }

    pub fn with_smooth(mut self, is_smooth: bool) -> Self {
        self.is_smooth = is_smooth;
        self
    }

    pub fn with_repeated(mut self, is_repeated: bool) -> Self {
        self.is_repeated = is_repeated;
        self
    }

    #[run_after(component(Renderer))]
    fn update(&mut self, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.texture = None;
            self.handler.reset();
        }
        if let Some(context) = state.context() {
            self.handler.update::<Self>(&self.key);
            if let Some(image) = self.handler.resource() {
                self.texture = Some(self.load_texture(image.0, context));
            }
        }
    }

    pub fn size(&self) -> Option<Size> {
        self.texture.as_ref().map(|t| t.size.into())
    }

    pub fn set_source(&mut self, source: TextureSource) {
        self.handler.set_source(source.into());
    }

    pub(crate) fn inner(&self) -> &LoadedTexture {
        self.texture
            .as_ref()
            .expect("internal error: not loaded texture")
    }

    fn load_texture(&mut self, image: RgbaImage, context: &GpuContext) -> LoadedTexture {
        let texture = self.create_texture(&image, context);
        Self::write_texture(&image, &texture, context);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = self.create_sampler(context);
        LoadedTexture {
            texture,
            size: Size::new(image.width(), image.height()).into(),
            bind_group: self.create_bind_group(&view, &sampler, context),
            is_transparent: image.pixels().any(|p| p.0[3] > 0 && p.0[3] < 255),
        }
    }

    fn create_texture(&self, image: &RgbaImage, context: &GpuContext) -> wgpu::Texture {
        context.device.create_texture(&TextureDescriptor {
            label: Some(&format!("modor_texture_{:?}", self.key)),
            size: Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: context
                .surface_texture_format
                .unwrap_or(Shader::DEFAULT_TEXTURE_FORMAT),
            usage: TextureUsages::TEXTURE_BINDING // for attachment to models
                | TextureUsages::COPY_DST // for attachment to models
                | TextureUsages::RENDER_ATTACHMENT // for rendering
                | TextureUsages::COPY_SRC, // for rendering
        })
    }

    fn create_sampler(&self, context: &GpuContext) -> Sampler {
        let address_mode = if self.is_repeated {
            AddressMode::Repeat
        } else {
            AddressMode::ClampToEdge
        };
        context.device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("modor_texture_sampler_{:?}", self.key)),
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
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
        context: &GpuContext,
    ) -> BindGroup {
        context.device.create_bind_group(&BindGroupDescriptor {
            layout: &context.texture_bind_group_layout,
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

    fn write_texture(image: &RgbaImage, texture: &wgpu::Texture, context: &GpuContext) {
        context.queue.write_texture(
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
        self.handler.state()
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum TextureSource {
    Size(Size),
    File(&'static [u8]),
    Path(String),
    RgbaBuffer(Vec<u8>, Size),
}

impl From<TextureSource> for ResourceSource<TextureData> {
    fn from(source: TextureSource) -> Self {
        match source {
            TextureSource::Size(size) => Self::AsyncData(TextureData::Size(size)),
            TextureSource::File(data) => Self::AsyncData(TextureData::File(data)),
            TextureSource::Path(path) => Self::AsyncPath(path),
            TextureSource::RgbaBuffer(buffer, size) => {
                Self::SyncData(TextureData::RgbaBuffer(buffer, size))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct LoadedTexture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) size: NonZeroSize,
    pub(crate) bind_group: BindGroup,
    pub(crate) is_transparent: bool,
}

#[derive(Debug, Clone)]
enum TextureData {
    Size(Size),
    File(&'static [u8]),
    RgbaBuffer(Vec<u8>, Size),
}

#[derive(Debug)]
struct LoadedImage(RgbaImage);

impl LoadedImage {
    fn load_from_memory(data: &[u8]) -> Result<Self, ResourceLoadingError> {
        image::load_from_memory(data)
            .map_err(Self::convert_error)
            .map(DynamicImage::into_rgba8)
            .map(Self)
    }

    fn load_from_buffer(buffer: Vec<u8>, size: Size) -> Result<Self, ResourceLoadingError> {
        let buffer_len = buffer.len();
        RgbaImage::from_raw(size.width, size.height, buffer)
            .ok_or_else(|| {
                ResourceLoadingError::InvalidFormat(format!(
                    "RGBA buffer size ({buffer_len}) does not correspond \
                    to specified image size ({size:?})",
                ))
            })
            .map(Self)
    }

    fn convert_error(error: ImageError) -> ResourceLoadingError {
        match error {
            ImageError::Decoding(e) => ResourceLoadingError::InvalidFormat(format!("{e}")),
            ImageError::Unsupported(e) => ResourceLoadingError::InvalidFormat(format!("{e}")),
            // coverage: off (internal errors that shouldn't happen)
            ImageError::Limits(_)
            | ImageError::Parameter(_)
            | ImageError::IoError(_)
            | ImageError::Encoding(_) => {
                ResourceLoadingError::LoadingError(format!("error when reading texture: {error}"))
            } // coverage: on
        }
    }
}

impl Load<TextureData> for LoadedImage {
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError> {
        Self::load_from_memory(&data)
    }

    fn load_from_data(data: &TextureData) -> Result<Self, ResourceLoadingError> {
        match data {
            TextureData::Size(size) => Ok(Self(RgbaImage::from_pixel(
                size.width.max(1),
                size.height.max(1),
                Rgba([255u8, 255, 255, 255]),
            ))),
            TextureData::File(data) => Self::load_from_memory(data),
            TextureData::RgbaBuffer(buffer, size) => Self::load_from_buffer(buffer.clone(), *size),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TextureKey {
    White,
    Invisible,
}
