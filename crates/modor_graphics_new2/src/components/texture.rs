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
            handler: ResourceHandler::new(source.into()),
            texture: None,
            renderer_version: None,
        }
    }

    pub fn with_smooth(mut self, is_smooth: bool) -> Self {
        self.is_smooth = is_smooth;
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
        context.device.create_sampler(&SamplerDescriptor {
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
    Unit,
    Size(Size),
    StaticData(&'static [u8]),
    Data(Vec<u8>),
    Path(String),
}

impl From<TextureSource> for ResourceSource<TextureData> {
    fn from(source: TextureSource) -> Self {
        match source {
            TextureSource::Unit => Self::SyncData(TextureData::Size(Size::new(1, 1))),
            TextureSource::Size(size) => Self::AsyncData(TextureData::Size(size)),
            TextureSource::StaticData(data) => Self::AsyncData(TextureData::StaticData(data)),
            TextureSource::Data(data) => Self::AsyncData(TextureData::Data(data)),
            TextureSource::Path(path) => Self::AsyncPath(path),
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
    StaticData(&'static [u8]),
    Data(Vec<u8>),
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
                size.width,
                size.height,
                Rgba([255u8, 255, 255, 255]),
            ))),
            TextureData::StaticData(data) => Self::load_from_memory(data),
            TextureData::Data(data) => Self::load_from_memory(data),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TextureKey {
    Blank,
}
