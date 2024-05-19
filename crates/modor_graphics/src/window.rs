use crate::gpu::{Gpu, GpuManager};
use crate::size::NonZeroSize;
use crate::{Size, Target};
use modor::{Context, Node, RootNode, Visit};
use std::mem;
use std::sync::Arc;
use wgpu::{Instance, Surface, SurfaceConfiguration, TextureFormat, TextureViewDescriptor};

#[derive(Visit)]
pub struct Window {
    /// Title of the window.
    ///
    /// Default is `""`.
    pub title: String,
    /// Whether the mouse cursor is shown when it is in the window.
    ///
    /// Default is `true`.
    pub is_cursor_visible: bool,
    pub target: Target,
    handle: Option<Arc<winit::window::Window>>,
    surface: WindowSurfaceState,
    old_state: OldWindowState,
}

impl RootNode for Window {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            title: String::new(),
            is_cursor_visible: true,
            target: Target::new(ctx, "window(modor_graphics)".into()),
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
    /// Default size is 800x600.
    ///
    /// If the app is not run with [`run`](crate::run), `None` is returned.
    ///
    /// # Platform-specific
    ///
    /// - Android: default size is the size of the screen.
    /// - Other: default size is 800x600.
    pub fn size(&self) -> Option<Size> {
        self.handle
            .as_ref()
            .map(|handle| handle.inner_size().into())
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
        instance
            .create_surface(handle.clone())
            .expect("cannot create surface")
    }

    pub(crate) fn set_surface(&mut self, surface: Surface<'static>) {
        self.surface = WindowSurfaceState::Loading(surface);
        self.target.reset();
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
                self.old_state.is_cursor_visible = self.is_cursor_visible;
            }
        }
    }

    fn update_surface(&mut self, ctx: &mut Context<'_>) {
        let gpu = ctx.get_mut::<GpuManager>().get().clone();
        let size = self.size();
        if let Some(surface) = self.surface.take_new() {
            let size = size.expect("internal error: not configured window").into();
            let surface = WindowSurface::new(&gpu, surface, size);
            let texture_format = surface.surface_config.format;
            self.surface = WindowSurfaceState::Loaded(surface);
            self.target.init(ctx, &gpu, size, texture_format);
        }
        if let WindowSurfaceState::Loaded(surface) = &mut self.surface {
            let size = size.expect("internal error: not configured window").into();
            surface.update(&gpu, size);
            self.target
                .update(ctx, &gpu, size, surface.surface_config.format);
            surface.render(ctx, &gpu, &mut self.target);
        }
    }
}

struct OldWindowState {
    title: String,
    is_cursor_visible: bool,
}

impl Default for OldWindowState {
    fn default() -> Self {
        Self {
            title: String::new(),
            is_cursor_visible: true,
        }
    }
}

enum WindowSurfaceState {
    None,
    Loading(Surface<'static>),
    Loaded(WindowSurface),
}

impl WindowSurfaceState {
    fn take_new(&mut self) -> Option<Surface<'static>> {
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
}

impl WindowSurface {
    fn new(gpu: &Gpu, surface: Surface<'static>, size: NonZeroSize) -> Self {
        let surface_config = Self::create_surface_config(gpu, &surface, size);
        Self {
            surface,
            surface_config,
        }
    }

    fn update(&mut self, gpu: &Gpu, size: NonZeroSize) {
        let width = size.width.into();
        let height = size.height.into();
        if self.surface_config.width != width || self.surface_config.height != height {
            self.surface_config.width = width;
            self.surface_config.height = height;
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
}
