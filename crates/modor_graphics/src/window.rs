use crate::gpu::{Gpu, GpuManager};
use crate::size::NonZeroSize;
use crate::{platform, validation, Camera2D, FrameRate, Size, Target};
use enum_iterator::Sequence;
use modor::{Context, Node, RootNode, Visit};
use std::mem;
use std::sync::Arc;
use wgpu::{
    Extent3d, Instance, PresentMode, Surface, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

// coverage: off (window cannot be tested)

#[derive(Visit)]
/// The main window where rendering is performed.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// #[derive(Node, Visit)]
/// struct Root {
///     // ...
/// }
///
/// impl RootNode for Root {
///     fn on_create(ctx: &mut Context<'_>) -> Self {
///         let window = ctx.get_mut::<Window>();
///         window.title = "My App".into();
///         window.frame_rate = FrameRate::Unlimited;
///         window.target.background_color = Color::GRAY;
///         // enable maximum supported anti-aliasing
///         window.anti_aliasing = window
///             .supported_anti_aliasing_modes()
///             .iter()
///             .copied()
///             .max()
///             .unwrap_or_default();
///         Self {
///             // ...
///         }
///     }
/// }
/// ```
pub struct Window {
    /// Title of the window.
    ///
    /// Default is `""`.
    pub title: String,
    /// Whether the mouse cursor is shown when it is in the window.
    ///
    /// Default is `true`.
    pub is_cursor_visible: bool,
    /// Render target of the window.
    pub target: Target,
    /// The rendering frame rate limit.
    ///
    /// Default is [`FrameRate::VSync`](FrameRate::VSync).
    pub frame_rate: FrameRate,
    /// Default camera of the window.
    pub camera: Camera2D,
    /// Anti-aliasing mode.
    ///
    /// If the mode is not supported, then no anti-aliasing is applied.
    ///
    /// Default is [`AntiAliasingMode::None`].
    pub anti_aliasing: AntiAliasingMode,
    pub(crate) size: Size,
    supported_anti_aliasing_modes: Vec<AntiAliasingMode>,
    handle: Option<Arc<winit::window::Window>>,
    surface: WindowSurfaceState,
    old_state: OldWindowState,
}

impl RootNode for Window {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let target = Target::new(ctx, "window(modor_graphics)");
        let camera = Camera2D::new(ctx, "window(modor_graphics)", vec![target.glob().clone()]);
        Self {
            title: String::new(),
            is_cursor_visible: true,
            target,
            frame_rate: FrameRate::VSync,
            camera,
            anti_aliasing: AntiAliasingMode::None,
            size: Self::DEFAULT_SIZE,
            supported_anti_aliasing_modes: vec![AntiAliasingMode::None],
            handle: None,
            surface: WindowSurfaceState::None,
            old_state: OldWindowState::default(),
        }
    }
}

impl Node for Window {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        self.update_properties();
        self.update_surface(ctx);
    }
}

impl Window {
    pub(crate) const DEFAULT_SIZE: Size = Size::new(800, 600);

    /// Returns the size of the window, which is also the size of the surface where the rendering
    /// is performed.
    ///
    /// If the app is not run with [`run`](crate::run), default size is returned.
    ///
    /// # Platform-specific
    ///
    /// - Android: default size is the size of the screen.
    /// - Other: default size is 800x600.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns all supported [`AntiAliasingMode`].
    pub fn supported_anti_aliasing_modes(&self) -> &[AntiAliasingMode] {
        &self.supported_anti_aliasing_modes
    }

    pub(crate) fn prepare_rendering(&self) {
        if let Some(handle) = &self.handle {
            handle.request_redraw();
        }
    }

    pub(crate) fn create_surface(
        &mut self,
        instance: &Instance,
        handle: Option<winit::window::Window>,
    ) -> Surface<'static> {
        let handle = if let Some(handle) = handle {
            self.handle.insert(handle.into())
        } else {
            self.handle
                .as_ref()
                .expect("internal error: not configured window")
        };
        let surface = instance
            .create_surface(handle.clone())
            .expect("cannot create surface");
        surface
    }

    pub(crate) fn set_surface(&mut self, gpu: &Gpu, surface: Surface<'static>) {
        let size = self
            .surface_size()
            .expect("internal error: not configured window");
        let surface = WindowSurface::new(gpu, surface, size);
        let texture_format = surface.surface_config.format;
        self.surface = WindowSurfaceState::Loading(surface);
        self.supported_anti_aliasing_modes = Self::supported_modes(gpu, texture_format);
    }

    pub(crate) fn texture_format(&self) -> Option<TextureFormat> {
        if let WindowSurfaceState::Loaded(surface) = &self.surface {
            Some(surface.surface_config.format)
        } else {
            None
        }
    }

    fn update_properties(&mut self) {
        if let Some(handle) = &self.handle {
            if self.title != self.old_state.title {
                handle.set_title(&self.title);
                self.old_state.title = self.title.clone();
            }
            if self.is_cursor_visible != self.old_state.is_cursor_visible {
                handle.set_cursor_visible(self.is_cursor_visible);
                platform::update_canvas_cursor(handle, self.is_cursor_visible);
                self.old_state.is_cursor_visible = self.is_cursor_visible;
            }
        }
    }

    fn update_surface(&mut self, ctx: &mut Context<'_>) {
        let gpu = ctx.get_mut::<GpuManager>().get_or_init().clone();
        let size = self.surface_size();
        if let Some(surface) = self.surface.take_new() {
            let texture_format = surface.surface_config.format;
            self.target
                .enable(ctx, &gpu, surface.size, texture_format, self.anti_aliasing);
            self.surface = WindowSurfaceState::Loaded(surface);
        }
        if let WindowSurfaceState::Loaded(surface) = &mut self.surface {
            let size = size.expect("internal error: not configured window");
            surface.update(&gpu, size, self.frame_rate);
            if size != self.old_state.size || self.anti_aliasing != self.old_state.anti_aliasing {
                let texture_format = surface.surface_config.format;
                self.target
                    .enable(ctx, &gpu, size, texture_format, self.anti_aliasing);
                self.old_state.size = size;
                self.old_state.anti_aliasing = self.anti_aliasing;
                self.camera.update(ctx); // force camera update to avoid distortion
            }
            surface.render(ctx, &gpu, &mut self.target);
        }
    }

    fn surface_size(&self) -> Option<NonZeroSize> {
        let handle = self.handle.as_ref()?;
        let size = PhysicalSize::new(self.size.width, self.size.height);
        let surface_size = platform::surface_size(handle, size);
        Some(Size::new(surface_size.width, surface_size.height).into())
    }

    fn supported_modes(gpu: &Gpu, format: TextureFormat) -> Vec<AntiAliasingMode> {
        enum_iterator::all::<AntiAliasingMode>()
            .filter(|&mode| Self::is_anti_aliasing_mode_supported(gpu, format, mode))
            .collect()
    }

    fn is_anti_aliasing_mode_supported(
        gpu: &Gpu,
        format: TextureFormat,
        mode: AntiAliasingMode,
    ) -> bool {
        if mode == AntiAliasingMode::None {
            return true;
        }
        validation::validate_wgpu(gpu, || {
            gpu.device.create_texture(&TextureDescriptor {
                label: Some("modor_texture:msaa_check"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: mode.sample_count(),
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        })
        .is_ok()
    }
}

struct OldWindowState {
    title: String,
    is_cursor_visible: bool,
    size: NonZeroSize,
    anti_aliasing: AntiAliasingMode,
}

impl Default for OldWindowState {
    fn default() -> Self {
        Self {
            title: "winit window".into(),
            is_cursor_visible: true,
            size: Window::DEFAULT_SIZE.into(),
            anti_aliasing: AntiAliasingMode::None,
        }
    }
}

enum WindowSurfaceState {
    None,
    Loading(WindowSurface),
    Loaded(WindowSurface),
}

impl WindowSurfaceState {
    fn take_new(&mut self) -> Option<WindowSurface> {
        match mem::replace(self, Self::None) {
            Self::Loading(surface) => Some(surface),
            other @ (Self::None | Self::Loaded(_)) => {
                *self = other;
                None
            }
        }
    }
}

struct WindowSurface {
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    size: NonZeroSize,
}

impl WindowSurface {
    fn new(gpu: &Gpu, surface: Surface<'static>, size: NonZeroSize) -> Self {
        let surface_config = Self::create_surface_config(gpu, &surface, size);
        Self {
            surface,
            surface_config,
            size,
        }
    }

    fn update(&mut self, gpu: &Gpu, size: NonZeroSize, frame_rate: FrameRate) {
        let width = size.width.into();
        let height = size.height.into();
        let present_mode = frame_rate.present_mode(Self::has_immediate_mode(gpu, &self.surface));
        if self.surface_config.width != width
            || self.surface_config.height != height
            || self.surface_config.present_mode != present_mode
        {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface_config.present_mode = present_mode;
            self.surface.configure(&gpu.device, &self.surface_config);
        }
    }

    fn render(&self, ctx: &mut Context<'_>, gpu: &Gpu, target: &mut Target) {
        let texture = self
            .surface
            .get_current_texture()
            .expect("internal error: cannot retrieve surface texture");
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        target.render(ctx, gpu, view);
        texture.present();
    }

    fn create_surface_config(
        gpu: &Gpu,
        surface: &Surface<'_>,
        size: NonZeroSize,
    ) -> SurfaceConfiguration {
        let config = surface
            .get_default_config(&gpu.adapter, size.width.into(), size.height.into())
            .expect("internal error: not supported surface");
        surface.configure(&gpu.device, &config);
        config
    }

    fn has_immediate_mode(gpu: &Gpu, surface: &Surface<'_>) -> bool {
        surface
            .get_capabilities(&gpu.adapter)
            .present_modes
            .contains(&PresentMode::Immediate)
    }
}

/// An anti-aliasing mode.
///
/// # Examples
///
/// See [`Window`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Sequence)]
#[non_exhaustive]
pub enum AntiAliasingMode {
    /// Anti-aliasing is disabled.
    #[default]
    None,
    /// Multi-Sample Anti-Aliasing is enabled with 2 samples.
    MsaaX2,
    /// Multi-Sample Anti-Aliasing is enabled with 4 samples.
    MsaaX4,
    /// Multi-Sample Anti-Aliasing is enabled with 8 samples.
    MsaaX8,
    /// Multi-Sample Anti-Aliasing is enabled with 16 samples.
    MsaaX16,
}

impl AntiAliasingMode {
    /// Returns the number of samples applied for anti-aliasing.
    pub const fn sample_count(self) -> u32 {
        match self {
            Self::None => 1,
            Self::MsaaX2 => 2,
            Self::MsaaX4 => 4,
            Self::MsaaX8 => 8,
            Self::MsaaX16 => 16,
        }
    }
}
