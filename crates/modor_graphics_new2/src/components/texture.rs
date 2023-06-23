use crate::components::renderer::Renderer;
use crate::components::shader::Shader;
use crate::data::size::NonZeroSize;
use crate::{GpuContext, Size};
use image::{DynamicImage, ImageError, RgbaImage};
use modor::Single;
use modor_resources::{
    IntoResourceKey, Load, Resource, ResourceHandler, ResourceKey, ResourceLoadingError,
    ResourceRegistry, ResourceSource, ResourceState,
};
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d,
    FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler, SamplerDescriptor,
    TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub(crate) type TextureRegistry = ResourceRegistry<Texture>;

/// A texture that can be attached to a [`Material`](crate::Material).
///
/// # Requirements
///
/// The texture can be loaded only if:
/// - graphics [`module`](crate::module()) is initialized
///
/// # Related components
///
/// - [`Material`](crate::Material)
/// - [`RenderTarget`](crate::RenderTarget)
/// - [`TextureBuffer`](crate::TextureBuffer)
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics_new2::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// #
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with_child(Texture::from_path(TextureKey, "texture.png"))
///         .with_child(Material::new(MaterialKey).with_texture_key(TextureKey))
///         .with_child(sprite())
/// }
///
/// fn sprite() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new())
///         .with(Model::rectangle(MaterialKey).with_camera_key(CameraKey))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct CameraKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct TextureKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct MaterialKey;
/// ```
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
    /// Creates a new texture identified by a unique `key` and created from `source`.
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

    /// Creates a white texture identified by a unique `key` and created with a given `size`.
    ///
    /// This method is equivalent to [`Texture::new`] with [`TextureSource::Size`] source.
    pub fn from_size(key: impl IntoResourceKey, size: Size) -> Self {
        Self::new(key, TextureSource::Size(size))
    }

    /// Creates a new texture identified by a unique `key` and created with a given `size` and RGBA
    /// `buffer`.
    ///
    /// This method is equivalent to [`Texture::new`] with [`TextureSource::Buffer`] source.
    pub fn from_buffer(key: impl IntoResourceKey, size: Size, buffer: Vec<u8>) -> Self {
        Self::new(key, TextureSource::Buffer(size, buffer))
    }

    /// Creates a new texture identified by a unique `key` and created with given file `data`.
    ///
    /// This method is equivalent to [`Texture::new`] with [`TextureSource::File`] source.
    pub fn from_file(key: impl IntoResourceKey, data: &'static [u8]) -> Self {
        Self::new(key, TextureSource::File(data))
    }

    /// Creates a new texture identified by a unique `key` and created with a given file `path`.
    ///
    /// This method is equivalent to [`Texture::new`] with [`TextureSource::Path`] source.
    pub fn from_path(key: impl IntoResourceKey, path: impl Into<String>) -> Self {
        Self::new(key, TextureSource::Path(path.into()))
    }

    /// Returns the texture with a given `is_smooth`.
    ///
    /// If `true`, a linear sampling is applied to the texture when it appears larger than its
    /// original size.
    ///
    /// Default is `true`.
    pub fn with_smooth(mut self, is_smooth: bool) -> Self {
        self.is_smooth = is_smooth;
        self
    }

    /// Returns the texture with a given `is_repeated`.
    ///
    /// If `true`, the texture is repeated when the texture width or height configured in an
    /// associated [`Material`](crate::Material) is greater than `1.0`.
    ///
    /// Default is `false`.
    pub fn with_repeated(mut self, is_repeated: bool) -> Self {
        self.is_repeated = is_repeated;
        self
    }

    #[run_after(component(Renderer))]
    fn update(&mut self, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.texture = None;
            self.handler.reload();
        }
        if let Some(context) = state.context() {
            self.handler.update::<Self>(&self.key);
            if let Some(image) = self.handler.resource() {
                self.texture = Some(self.load_texture(image.0, context));
            }
        }
    }

    /// Returns the size of the texture.
    ///
    /// [`None`] is returned if the texture is not loaded.
    pub fn size(&self) -> Option<Size> {
        self.texture.as_ref().map(|t| t.size.into())
    }

    /// Sets the texture `source` and start reloading of the texture.
    ///
    /// If the previous source is already loaded, the texture remains valid until the new source
    /// is loaded.
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
            format: Shader::TEXTURE_FORMAT,
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
        if self.texture.is_some() {
            ResourceState::Loaded
        } else {
            self.handler.state()
        }
    }
}

/// The source of a [`Texture`].
///
/// Sources loaded synchronously are ready after the next [`App`](modor::App) update. Sources loaded
/// asynchronously can take more updates to be ready.
///
/// # Examples
///
/// See [`Texture`].
#[non_exhaustive]
#[derive(Debug)]
pub enum TextureSource {
    /// White texture created synchronously with a given size.
    ///
    /// If width or height is zero, then the created texture will have a size of 1x1.
    Size(Size),
    /// Texture loaded synchronously from given size and RGBA buffer.
    ///
    /// If width or height is zero, then a white texture is created with size 1x1.
    Buffer(Size, Vec<u8>),
    /// Texture loaded asynchronously from given file bytes.
    ///
    /// This variant is generally used in combination with [`include_bytes!`].
    File(&'static [u8]),
    /// Texture loaded asynchronously from a given path.
    ///
    /// # Platform-specific
    ///
    /// - Web: HTTP GET call is performed to retrieve the file from URL
    /// `{current_browser_url}/assets/{path}`.
    /// - Android: the file is retrieved using the Android
    /// [`AssetManager`](https://developer.android.com/reference/android/content/res/AssetManager).
    /// - Other: if `CARGO_MANIFEST_DIR` environment variable is set (this is the case if the
    /// application is run using a `cargo` command), then the file is retrieved from path
    /// `{CARGO_MANIFEST_DIR}/assets/{path}`. Else, the file path is
    /// `{executable_folder_path}/assets/{path}`.
    Path(String),
}

impl From<TextureSource> for ResourceSource<TextureData> {
    fn from(source: TextureSource) -> Self {
        match source {
            TextureSource::Size(size) => {
                let size = NonZeroSize::from(size);
                Self::SyncData(TextureData::RgbaBuffer(
                    size,
                    vec![255; (u32::from(size.width) * u32::from(size.height) * 4) as usize],
                ))
            }
            TextureSource::Buffer(size, buffer) => {
                let buffer = if buffer.is_empty() {
                    vec![255; 4]
                } else {
                    buffer
                };
                Self::SyncData(TextureData::RgbaBuffer(NonZeroSize::from(size), buffer))
            }
            TextureSource::File(data) => Self::AsyncData(TextureData::File(data)),
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
    File(&'static [u8]),
    RgbaBuffer(NonZeroSize, Vec<u8>),
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

    fn load_from_buffer(buffer: Vec<u8>, size: NonZeroSize) -> Result<Self, ResourceLoadingError> {
        let buffer_len = buffer.len();
        RgbaImage::from_raw(u32::from(size.width), u32::from(size.height), buffer)
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
            TextureData::File(data) => Self::load_from_memory(data),
            TextureData::RgbaBuffer(size, buffer) => Self::load_from_buffer(buffer.clone(), *size),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum TextureKey {
    White,
    Invisible,
}
