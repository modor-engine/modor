use crate::anti_aliasing::SupportedAntiAliasingModes;
use crate::gpu::{Gpu, GpuManager};
use crate::size::NonZeroSize;
use crate::texture::internal::TextureLoaded;
use crate::{Camera2D, Size, Target};
use glob::TextureGlob;
use image::{DynamicImage, RgbaImage};
use modor::{App, Builder, FromApp, Glob};
use modor_resources::{ResSource, Resource, ResourceError, Source};
use wgpu::{TextureFormat, TextureViewDescriptor};

/// A texture that can be attached to a [material](crate::Mat).
///
/// # Examples
///
/// ```rust
/// # use wgpu::core::command::CopySide::Source;
/// use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::modor_resources::*;
/// # use modor_physics::modor_math::*;
/// #
/// struct TexturedRectangle {
///     sprite: Sprite2D,
/// }
///
/// impl TexturedRectangle {
///     fn new(app: &mut App, position: Vec2, size: Vec2) -> Self {
///         let  resources = app.get_mut::<Resources>();
///         let camera = resources.target.camera.glob().to_ref();
///         let texture = resources.texture.glob().to_ref();
///         Self {
///             sprite: Sprite2D::new(app)
///                 .with_material(|m| m.texture = texture)
///                 .with_model(|m| m.position = position)
///                 .with_model(|m| m.size = size)
///                 .with_model(|m| m.camera = camera),
///         }
///     }
///
///     fn update(&mut self, app: &mut App) {
///         self.sprite.update(app);
///     }
/// }
///
/// struct Resources {
///     texture: Res<Texture>,
///     target: Res<Texture>,
/// }
///
/// impl FromApp for Resources {
///     fn from_app(app: &mut App) -> Self {
///         Self {
///             texture: Texture::new(app).load_from_path(app, "my-texture.png"),
///             target: Texture::new(app)
///                 .with_is_target_enabled(true)
///                 .load_from_source(app, TextureSource::Size(Size::new(800, 600))),
///         }
///     }
/// }
///
/// impl RootNode for Resources {
///     fn update(&mut self, app: &mut App) {
///         self.texture.update(app);
///         self.target.update(app);
///     }
/// }
/// ```
#[derive(Debug, Builder)]
#[allow(clippy::struct_excessive_bools)]
pub struct Texture {
    /// Whether the texture is smooth.
    ///
    /// If `true`, a linear sampling is applied to the texture when it is rendered larger than its
    /// original size.
    ///
    /// Default is `true`.
    #[builder(form(value))]
    pub is_smooth: bool,
    /// Whether the texture is repeated.
    ///
    /// If `true`, the texture is rendered repeated when the texture width or height configured in
    /// an associated [`Material`](crate::Material) is greater than `1.0`.
    ///
    /// Default is `false`.
    #[builder(form(value))]
    pub is_repeated: bool,
    /// Whether the texture buffer is enabled.
    ///
    /// The buffer can be used to retrieve pixels of the texture stored on GPU side.
    /// It is accessible with the [`TextureGlob`].
    ///
    /// Default is `false`.
    #[builder(form(value))]
    pub is_buffer_enabled: bool,
    /// Whether the texture is a rendering [`target`](#structfield.target).
    ///
    /// Default is `false`.
    #[builder(form(value))]
    pub is_target_enabled: bool,
    /// Render target of the texture.
    ///
    /// Doesn't have effect if [`is_target_enabled`](#structfield.is_target_enabled) is `false`.
    #[builder(form(closure))]
    pub target: Target,
    /// Default camera of the texture target.
    ///
    /// Doesn't have effect if [`is_target_enabled`](#structfield.is_target_enabled) is `false`.
    #[builder(form(closure))]
    pub camera: Camera2D,
    loaded: TextureLoaded,
    glob: Glob<TextureGlob>,
}

impl Resource for Texture {
    type Source = TextureSource;
    type Loaded = TextureLoaded;

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

    fn update(&mut self, app: &mut App, loaded: Option<Self::Loaded>, _source: &ResSource<Self>) {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        if let Some(loaded) = loaded {
            self.loaded = loaded;
            *self.glob.get_mut(app) = TextureGlob::new(
                app,
                &self.loaded,
                self.is_repeated,
                self.is_smooth,
                self.is_buffer_enabled,
            );
            let size = Size::new(self.loaded.image.width(), self.loaded.image.height()).into();
            self.init_target(app, &gpu, size);
        }
        self.glob.get_mut(app).update(
            &gpu,
            self.is_repeated,
            self.is_smooth,
            self.is_buffer_enabled,
        );
        self.update_target();
        self.camera.update(app);
        self.render_target(app, &gpu);
        self.glob.get_mut(app).update_buffer(&gpu);
    }
}

impl Texture {
    pub(crate) const DEFAULT_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub(crate) const DEFAULT_IS_SMOOTH: bool = true;
    pub(crate) const DEFAULT_IS_REPEATED: bool = false;
    pub(crate) const DEFAULT_IS_BUFFER_ENABLED: bool = false;

    /// Creates a new texture.
    pub fn new(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let mut target = Target::new(app);
        target.supported_anti_aliasing_modes = app
            .get_mut::<SupportedAntiAliasingModes>()
            .get(&gpu, Self::DEFAULT_FORMAT)
            .to_vec();
        let camera = Camera2D::new(app, vec![target.glob().to_ref()]);
        Self {
            is_smooth: Self::DEFAULT_IS_SMOOTH,
            is_repeated: Self::DEFAULT_IS_REPEATED,
            is_buffer_enabled: Self::DEFAULT_IS_BUFFER_ENABLED,
            is_target_enabled: false,
            target,
            camera,
            loaded: TextureLoaded::default(),
            glob: Glob::from_app(app),
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &Glob<TextureGlob> {
        &self.glob
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

    fn init_target(&mut self, app: &mut App, gpu: &Gpu, size: NonZeroSize) {
        if self.is_target_enabled {
            self.target.enable(app, gpu, size, Self::DEFAULT_FORMAT);
        }
    }

    fn update_target(&mut self) {
        if !self.is_target_enabled {
            self.target.disable();
        }
    }

    fn render_target(&mut self, app: &mut App, gpu: &Gpu) {
        if self.is_target_enabled {
            let view = self
                .glob
                .get_mut(app)
                .texture
                .create_view(&TextureViewDescriptor::default());
            self.target.render(app, gpu, view);
        }
    }
}

/// The source of a [`Texture`].
///
/// # Examples
///
/// See [`Texture`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TextureSource {
    /// White texture created synchronously with a given size.
    ///
    /// If width or height is zero, then the texture is created with size 1x1.
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

pub(crate) mod glob;
