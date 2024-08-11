use crate::anti_aliasing::SupportedAntiAliasingModes;
use crate::gpu::{Gpu, GpuManager};
use crate::size::NonZeroSize;
use crate::texture::internal::TextureLoaded;
use crate::{AntiAliasingMode, Camera2D, Size, Target, TextureGlob};
use getset::CopyGetters;
use image::{DynamicImage, RgbaImage};
use modor::{App, FromApp, Glob, Globals, State, Update, Updater};
use modor_resources::{Res, ResSource, ResUpdater, Resource, ResourceError, Source};
use std::marker::PhantomData;
use wgpu::{TextureFormat, TextureViewDescriptor};

pub(crate) mod glob;

/// A texture that can be attached to a [material](crate::Mat).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
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
///         let (camera, texture) = app.take::<Resources, _>(|resources, app| (
///             resources.target.get(app).camera.glob().to_ref(),
///             resources.texture.to_ref(),
///         ));
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
/// #[derive(FromApp)]
/// struct Resources {
///     texture: Glob<Res<Texture>>,
///     target: Glob<Res<Texture>>,
/// }
///
/// impl State for Resources {
///     fn init(&mut self, app: &mut App) {
///         TextureUpdater::default()
///             .res(ResUpdater::default().path("my-texture.png"))
///             .apply(app, &self.texture);
///         TextureUpdater::default()
///             .res(ResUpdater::default().source(TextureSource::Size(Size::new(800, 600))))
///             .is_target_enabled(true)
///             .apply(app, &self.target);
///     }
/// }
/// ```
#[derive(Debug, Updater, CopyGetters)]
#[allow(clippy::struct_excessive_bools)]
pub struct Texture {
    /// Whether the texture is smooth.
    ///
    /// If `true`, a linear sampling is applied to the texture when it is rendered larger than its
    /// original size.
    ///
    /// Default is `true`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_smooth: bool,
    /// Whether the texture is repeated.
    ///
    /// If `true`, the texture is rendered repeated when the texture width or height configured in
    /// an associated [`Material`](crate::Material) is greater than `1.0`.
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_repeated: bool,
    /// Whether the texture buffer is enabled.
    ///
    /// The buffer can be used to retrieve pixels of the texture stored on GPU side.
    /// It is accessible with the [`TextureGlob`].
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_buffer_enabled: bool,
    /// Whether the texture is a rendering [`target`](#structfield.target).
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_target_enabled: bool,
    #[updater(inner_type, field, for_field)]
    target_anti_aliasing: PhantomData<AntiAliasingMode>,
    #[updater(inner_type, field)]
    res: PhantomData<ResUpdater<Texture>>,
    /// Render target of the texture.
    ///
    /// Doesn't have effect if [`is_target_enabled`](#structfield.is_target_enabled) is `false`.
    // TODO: add updater
    pub target: Target,
    /// Default camera of the texture target.
    ///
    /// Doesn't have effect if [`is_target_enabled`](#structfield.is_target_enabled) is `false`.
    // TODO: add updater
    pub camera: Camera2D,
    loaded: TextureLoaded,
    pub glob: TextureGlob,
}

impl FromApp for Texture {
    fn from_app(app: &mut App) -> Self {
        app.create::<TextureManager>();
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
            target_anti_aliasing: PhantomData,
            res: PhantomData,
            target,
            camera,
            loaded: TextureLoaded::default(),
            glob: TextureGlob::from_app(app),
        }
    }
}

impl Resource for Texture {
    type Source = TextureSource;
    type Loaded = TextureLoaded;

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        Self::load_from_file(&file_bytes).map(Into::into)
    }

    fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        Ok(TextureLoaded::from(match source {
            TextureSource::Size(size) => Self::load_from_size(*size, None)?,
            TextureSource::Buffer(size, buffer) => Self::load_from_size(*size, Some(buffer))?,
            TextureSource::Bytes(bytes) => Self::load_from_file(bytes)?,
        }))
    }

    fn on_load(&mut self, app: &mut App, loaded: Self::Loaded, _source: &ResSource<Self>) {
        self.loaded = loaded;
        self.glob = TextureGlob::new(
            app,
            &self.loaded,
            self.is_repeated,
            self.is_smooth,
            self.is_buffer_enabled,
        );
        self.update(app);
    }
}

impl Texture {
    pub(crate) const DEFAULT_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub(crate) const DEFAULT_IS_SMOOTH: bool = true;
    pub(crate) const DEFAULT_IS_REPEATED: bool = false;
    pub(crate) const DEFAULT_IS_BUFFER_ENABLED: bool = false;

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

    fn update(&mut self, app: &mut App) {
        self.glob.update(
            app.get_mut::<GpuManager>().get_or_init(),
            self.is_repeated,
            self.is_smooth,
            self.is_buffer_enabled,
        );
        if self.is_target_enabled {
            let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
            let size = self.glob.size.into();
            self.target.enable(app, &gpu, size, Texture::DEFAULT_FORMAT);
        } else {
            self.target.disable();
        }
    }

    fn render(&mut self, app: &mut App) {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        self.camera.update(app);
        self.render_target(app, &gpu);
        self.glob.update_buffer(&gpu);
    }

    fn render_target(&mut self, app: &mut App, gpu: &Gpu) {
        if self.is_target_enabled {
            let view = self
                .glob
                .texture
                .create_view(&TextureViewDescriptor::default());
            self.target.render(app, gpu, view);
        }
    }
}

impl TextureUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &Glob<Res<Texture>>) {
        glob.take(app, |tex, app| {
            Update::apply(
                &mut self.target_anti_aliasing,
                &mut tex.target.anti_aliasing,
            );
            if Update::apply_checked(&mut self.is_smooth, &mut tex.is_smooth)
                | Update::apply_checked(&mut self.is_repeated, &mut tex.is_repeated)
                | Update::apply_checked(&mut self.is_buffer_enabled, &mut tex.is_buffer_enabled)
                | Update::apply_checked(&mut self.is_target_enabled, &mut tex.is_target_enabled)
            {
                tex.update(app);
            }
        });
        if let Some(res) = self.res.take_value(|| unreachable!()) {
            res.apply(app, glob);
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

// TODO: remove if possible
#[derive(FromApp)]
struct TextureManager;

impl State for TextureManager {
    fn update(&mut self, app: &mut App) {
        app.take::<Globals<Res<Texture>>, _>(|textures, app| {
            for texture in textures.iter_mut() {
                texture.render(app);
            }
        });
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
