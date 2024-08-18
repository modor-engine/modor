use crate::anti_aliasing::SupportedAntiAliasingModes;
use crate::gpu::{Gpu, GpuManager};
use crate::material::MaterialManager;
use crate::size::NonZeroSize;
use crate::texture::internal::TextureLoaded;
use crate::{AntiAliasingMode, Camera2D, Color, Size, Target};
use getset::{CopyGetters, Getters};
use image::{DynamicImage, RgbaImage};
use modor::{App, FromApp, Glob, GlobRef, Globals, State, StateHandle, Update, Updater};
use modor_input::modor_math::Vec2;
use modor_resources::{Res, ResSource, ResUpdater, Resource, ResourceError, Source};
use std::marker::PhantomData;
use std::mem;
use std::num::NonZeroU32;
use wgpu::{
    AddressMode, Buffer, BufferView, CommandEncoderDescriptor, Extent3d, FilterMode,
    ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, MapMode, Origin3d, Sampler,
    SamplerDescriptor, SubmissionIndex, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

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
///             resources.target.get(app).camera().glob().to_ref(),
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
#[derive(Debug, Updater, CopyGetters, Getters)]
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
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_buffer_enabled: bool,
    /// Whether the texture is a rendering [`target`](Texture::target).
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_target_enabled: bool,
    /// Anti-aliasing mode of the texture target.
    ///
    /// If the mode is not supported, then no anti-aliasing is applied.
    ///
    /// Default is [`AntiAliasingMode::None`].
    #[updater(inner_type, field, for_field)]
    target_anti_aliasing: PhantomData<AntiAliasingMode>,
    /// Background color used for texture target rendering.
    ///
    /// Default is [`Color::BLACK`].
    #[updater(inner_type, field, for_field)]
    target_background_color: PhantomData<Color>,
    /// Position of the default camera rendered zone center in world units.
    ///
    /// Doesn't have effect if [`is_target_enabled`](Texture::is_target_enabled) is `false`.
    #[updater(inner_type, field, for_field)]
    camera_position: PhantomData<Vec2>,
    /// Size of the default camera rendered zone in world units.
    ///
    /// Doesn't have effect if [`is_target_enabled`](Texture::is_target_enabled) is `false`.
    #[updater(inner_type, field, for_field)]
    camera_size: PhantomData<Vec2>,
    /// Rotation in radians of the default camera around its position.
    ///
    /// Doesn't have effect if [`is_target_enabled`](Texture::is_target_enabled) is `false`.
    #[updater(inner_type, field, for_field)]
    camera_rotation: PhantomData<f32>,
    /// The render targets where the default camera should be used.
    ///
    /// If a camera is linked to a target, then all models linked to the camera are rendered in the
    /// target.
    ///
    /// Doesn't have effect if [`is_target_enabled`](Texture::is_target_enabled) is `false`.
    #[updater(inner_type, field, for_field)]
    camera_targets: PhantomData<Vec<GlobRef<Target>>>,
    /// General resource parameters.
    #[updater(inner_type, field)]
    res: PhantomData<ResUpdater<Texture>>,
    /// Render target of the texture.
    ///
    /// Doesn't have effect if [`is_target_enabled`](Texture::is_target_enabled) is `false`.
    #[getset(get = "pub")]
    target: Glob<Target>,
    /// Default camera of the texture target.
    ///
    /// Doesn't have effect if [`is_target_enabled`](Texture::is_target_enabled) is `false`.
    #[getset(get = "pub")]
    camera: Camera2D,
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    pub(super) texture: wgpu::Texture,
    pub(crate) loaded: TextureLoaded,
    buffer: Option<Buffer>,
    submission_index: Option<SubmissionIndex>,
    gpu_manager: StateHandle<GpuManager>,
}

impl FromApp for Texture {
    fn from_app(app: &mut App) -> Self {
        app.create::<TextureManager>();
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let target = Glob::<Target>::from_app(app);
        target.get_mut(app).supported_anti_aliasing_modes = app
            .get_mut::<SupportedAntiAliasingModes>()
            .get(&gpu, Self::DEFAULT_FORMAT)
            .to_vec();
        let camera = Camera2D::new(app, vec![target.to_ref()]);
        let loaded = TextureLoaded::default();
        let texture = Self::create_texture(&gpu, &loaded);
        Self::write_texture(&gpu, &loaded, &texture);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler =
            Self::create_sampler(&gpu, Self::DEFAULT_IS_REPEATED, Self::DEFAULT_IS_SMOOTH);
        Self {
            is_smooth: Self::DEFAULT_IS_SMOOTH,
            is_repeated: Self::DEFAULT_IS_REPEATED,
            is_buffer_enabled: Self::DEFAULT_IS_BUFFER_ENABLED,
            is_target_enabled: false,
            target_anti_aliasing: PhantomData,
            target_background_color: PhantomData,
            camera_position: PhantomData,
            camera_size: PhantomData,
            camera_rotation: PhantomData,
            camera_targets: PhantomData,
            res: PhantomData,
            target,
            camera,
            loaded,
            view,
            sampler,
            texture,
            buffer: None,
            submission_index: None,
            gpu_manager: app.handle(),
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

    fn on_load(
        &mut self,
        app: &mut App,
        index: usize,
        loaded: Self::Loaded,
        _source: &ResSource<Self>,
    ) {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        self.loaded = loaded;
        self.texture = Self::create_texture(&gpu, &self.loaded);
        Self::write_texture(&gpu, &self.loaded, &self.texture);
        self.view = self.texture.create_view(&TextureViewDescriptor::default());
        self.sampler = Self::create_sampler(&gpu, self.is_repeated, self.is_smooth);
        self.submission_index = None;
        self.update(app, true, index);
        self.copy_texture_in_buffer(&gpu);
    }
}

impl Texture {
    pub(crate) const DEFAULT_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    const DEFAULT_IS_SMOOTH: bool = true;
    const DEFAULT_IS_REPEATED: bool = false;
    const DEFAULT_IS_BUFFER_ENABLED: bool = false;
    const COMPONENT_COUNT_PER_PIXEL: u32 = 4;

    /// Returns the size of the texture in pixels.
    pub fn size(&self) -> Size {
        Size::new(self.loaded.image.width(), self.loaded.image.height())
    }

    /// Retrieves the texture buffer from the GPU.
    ///
    /// Each item is the component value of a pixel, and each pixel has 4 components (RGBA format).
    ///
    /// The returned buffer contains data only if:
    /// - The [`Texture`] buffer is enabled.
    /// - The [`Texture`] buffer has been updated.
    ///
    /// Note that retrieving data from the GPU may have a significant impact on performance.
    pub fn buffer(&self, app: &App) -> Vec<u8> {
        let gpu = self
            .gpu_manager
            .get(app)
            .get()
            .expect("internal error: not initialized GPU");
        if let (Some(buffer), Some(submission_index)) = (&self.buffer, &self.submission_index) {
            let view = Self::buffer_view(gpu, buffer, submission_index);
            let data = self.retrieve_buffer(&view);
            drop(view);
            buffer.unmap();
            data
        } else {
            vec![]
        }
    }

    /// Retrieves a pixel color from the GPU.
    ///
    /// The color is returned only if:
    /// - The [`Texture`] buffer is enabled.
    /// - The [`Texture`] buffer has been updated.
    /// - The pixel coordinates (`x`, `y`) are not out of bound.
    ///
    /// Note that retrieving data from the GPU may have a significant impact on performance.
    pub fn color(&self, app: &App, x: u32, y: u32) -> Option<Color> {
        let gpu = self
            .gpu_manager
            .get(app)
            .get()
            .expect("internal error: not initialized GPU");
        if let (Some(buffer), Some(submission_index)) = (&self.buffer, &self.submission_index) {
            let view = Self::buffer_view(gpu, buffer, submission_index);
            let color = self.retrieve_pixel_color(x, y, &view);
            drop(view);
            buffer.unmap();
            color
        } else {
            None
        }
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

    fn update(&mut self, app: &mut App, is_reloaded: bool, texture_index: usize) {
        let gpu = app.get_mut::<GpuManager>().get_or_init();
        self.sampler = Self::create_sampler(gpu, self.is_repeated, self.is_smooth);
        if (self.buffer.is_none() || is_reloaded) && self.is_buffer_enabled {
            self.buffer = Some(Self::create_buffer(gpu, self.size()));
        } else if self.buffer.is_some() && !self.is_buffer_enabled {
            self.buffer = None;
        }
        if self.is_target_enabled {
            let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
            let size = self.size().into();
            self.target
                .get_mut(app)
                .enable(&gpu, size, Self::DEFAULT_FORMAT);
        } else {
            self.target.get_mut(app).disable();
        }
        app.get_mut::<MaterialManager>()
            .register_loaded_texture(texture_index);
    }

    fn prepare_rendering(&mut self, app: &mut App) -> (GlobRef<Target>, TextureView) {
        self.camera.update(app);
        (
            self.target.to_ref(),
            self.texture.create_view(&TextureViewDescriptor::default()),
        )
    }

    fn create_texture(gpu: &Gpu, loaded: &TextureLoaded) -> wgpu::Texture {
        gpu.device.create_texture(&TextureDescriptor {
            label: Some("modor_texture"),
            size: Extent3d {
                width: loaded.image.width(),
                height: loaded.image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEFAULT_FORMAT,
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

    fn create_sampler(gpu: &Gpu, is_repeated: bool, is_smooth: bool) -> Sampler {
        let address_mode = if is_repeated {
            AddressMode::Repeat
        } else {
            AddressMode::ClampToEdge
        };
        gpu.device.create_sampler(&SamplerDescriptor {
            label: Some("modor_texture_sampler"),
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

    fn create_buffer(gpu: &Gpu, size: Size) -> Buffer {
        let padded_bytes_per_row = Self::calculate_padded_row_bytes(size.width);
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modor_texture_buffer"),
            size: u64::from(padded_bytes_per_row * size.height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn copy_texture_in_buffer(&mut self, gpu: &Gpu) {
        let Some(buffer) = &self.buffer else {
            return;
        };
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size().width);
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_texture_buffer_encoder"),
        };
        let mut encoder = gpu.device.create_command_encoder(&descriptor);
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            ImageCopyBuffer {
                buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        NonZeroU32::new(padded_row_bytes)
                            .expect("internal error: cannot render empty buffer")
                            .into(),
                    ),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: self.size().width,
                height: self.size().height,
                depth_or_array_layers: 1,
            },
        );
        self.submission_index = Some(gpu.queue.submit(Some(encoder.finish())));
    }

    fn buffer_view<'a>(
        gpu: &Gpu,
        buffer: &'a Buffer,
        submission_index: &SubmissionIndex,
    ) -> BufferView<'a> {
        let slice = buffer.slice(..);
        slice.map_async(MapMode::Read, |_| ());
        gpu.device.poll(wgpu::Maintain::WaitForSubmissionIndex(
            submission_index.clone(),
        ));
        slice.get_mapped_range()
    }

    fn retrieve_buffer(&self, view: &BufferView<'_>) -> Vec<u8> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size().width);
        let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(self.size().width);
        let data = view
            .chunks(padded_row_bytes as usize)
            .flat_map(|a| &a[..unpadded_row_bytes as usize])
            .copied()
            .collect();
        data
    }

    fn retrieve_pixel_color(&self, x: u32, y: u32, view: &BufferView<'_>) -> Option<Color> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size().width);
        let color_start = y * padded_row_bytes + Self::COMPONENT_COUNT_PER_PIXEL * x;
        Self::extract_color(view, color_start)
    }

    fn extract_color(data: &[u8], start_index: u32) -> Option<Color> {
        if start_index as usize >= data.len() {
            return None;
        }
        let end_index = start_index + Self::COMPONENT_COUNT_PER_PIXEL;
        let color = &data[start_index as usize..end_index as usize];
        Some(Color::rgba(
            f32::from(color[0]) / 255.,
            f32::from(color[1]) / 255.,
            f32::from(color[2]) / 255.,
            f32::from(color[3]) / 255.,
        ))
    }

    fn calculate_padded_row_bytes(width: u32) -> u32 {
        let unpadded_bytes_per_row = Self::calculate_unpadded_row_bytes(width);
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padded_bytes_per_row_padding
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_unpadded_row_bytes(width: u32) -> u32 {
        let bytes_per_pixel = mem::size_of::<u32>() as u32;
        width * bytes_per_pixel
    }
}

impl TextureUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &Glob<Res<Texture>>) {
        glob.take(app, |tex, app| {
            Update::apply(
                &mut self.target_anti_aliasing,
                &mut tex.target.get_mut(app).anti_aliasing,
            );
            Update::apply(
                &mut self.target_background_color,
                &mut tex.target.get_mut(app).background_color,
            );
            Update::apply(&mut self.camera_position, &mut tex.camera.position);
            Update::apply(&mut self.camera_size, &mut tex.camera.size);
            Update::apply(&mut self.camera_rotation, &mut tex.camera.rotation);
            Update::apply(&mut self.camera_targets, &mut tex.camera.targets);
            if Update::apply_checked(&mut self.is_smooth, &mut tex.is_smooth)
                | Update::apply_checked(&mut self.is_repeated, &mut tex.is_repeated)
                | Update::apply_checked(&mut self.is_buffer_enabled, &mut tex.is_buffer_enabled)
                | Update::apply_checked(&mut self.is_target_enabled, &mut tex.is_target_enabled)
            {
                tex.update(app, false, glob.index());
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

#[derive(FromApp)]
struct TextureManager;

impl State for TextureManager {
    fn update(&mut self, app: &mut App) {
        let texture_indexes = app
            .get_mut::<Globals<Res<Texture>>>()
            .iter_enumerated()
            .filter(|(_, texture)| texture.is_target_enabled)
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        for texture_index in texture_indexes {
            let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
            let (target, view) =
                Self::run_on_texture(app, texture_index, |t, app| t.prepare_rendering(app));
            target.take(app, |target, app| target.render(app, &gpu, view));
            Self::run_on_texture(app, texture_index, |t, _| t.copy_texture_in_buffer(&gpu));
        }
    }
}

impl TextureManager {
    fn run_on_texture<O>(
        app: &mut App,
        texture_index: usize,
        f: impl FnOnce(&mut Texture, &mut App) -> O,
    ) -> O {
        app.take::<Globals<Res<Texture>>, _>(|glob, app| {
            f(
                glob.get_mut(texture_index)
                    .expect("internal error: invalid texture index"),
                app,
            )
        })
    }
}

mod internal {
    use image::{Rgba, RgbaImage};

    #[derive(Debug)]
    pub struct TextureLoaded {
        pub(super) image: RgbaImage,
        pub(crate) is_transparent: bool,
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
