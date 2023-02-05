use crate::settings::frame_rate::FrameRate;
use crate::settings::frame_rate::FrameRateLimit;
use crate::settings::rendering::Resolution;
use crate::settings::window::{CursorVisibility, WindowTitle};
use crate::targets::GpuDevice;
use crate::targets::Target;
use futures::executor;
use modor::{Built, EntityBuilder, Single};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use wgpu::{
    Adapter, Backends, Instance, PowerPreference, PresentMode, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureUsages, TextureViewDescriptor,
};
use winit::dpi::PhysicalSize;

pub(crate) struct WindowTarget {
    instance: Instance,
    adapter: Adapter,
    surface: Surface,
    surface_config: SurfaceConfiguration,
    is_surface_refreshed: bool,
    is_surface_invalid: bool,
    current_texture: Option<SurfaceTexture>,
}

#[singleton]
impl WindowTarget {
    pub(crate) fn build(window: Arc<RwLock<winit::window::Window>>) -> impl Built<Self> {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all);
        let instance = Instance::new(backends);
        let window_ref = window
            .try_read()
            .expect("internal error: not readable window");
        let surface = Self::create_surface(&instance, &window_ref);
        let window_size = window_ref.inner_size();
        let adapter = Self::retrieve_adapter(&instance, &surface);
        let (device, queue) = super::retrieve_device_and_queue(&adapter);
        let (width, height) = (window_size.width.max(1), window_size.height.max(1));
        let surface_config = Self::create_surface_config(&surface, &adapter, width, height);
        let format = surface_config.format;
        surface.configure(&device, &surface_config);
        drop(window_ref);
        EntityBuilder::new(Self {
            instance,
            adapter,
            surface,
            surface_config,
            is_surface_refreshed: false,
            is_surface_invalid: false,
            current_texture: None,
        })
        .inherit_from(Target::build(device, queue, width, height, format))
        .with_child(Window::build(window))
    }

    #[run]
    fn reset_surface(&mut self, window: Single<'_, Window>) {
        if !self.is_surface_invalid {
            return;
        }
        let window_handle = window.handle();
        let PhysicalSize { width, height } = window_handle.inner_size();
        let (width, height) = (width.max(1), height.max(1));
        self.surface = Self::create_surface(&self.instance, &window.handle());
        self.surface_config =
            Self::create_surface_config(&self.surface, &self.adapter, width, height);
        self.is_surface_invalid = false;
        self.is_surface_refreshed = true;
    }

    #[run_after_previous]
    fn update_frame_rate(&mut self, frame_rate: Single<'_, FrameRate>) {
        let is_vsync = matches!(frame_rate.limit, FrameRateLimit::VSync);
        let present_mode = if is_vsync || !self.is_immediate_mode_supported() {
            PresentMode::Fifo
        } else {
            PresentMode::Immediate
        };
        if self.surface_config.present_mode != present_mode {
            self.surface_config.present_mode = present_mode;
            self.is_surface_refreshed = true;
        }
    }

    #[run_after_previous]
    fn update_resolution(&mut self, resolution: Single<'_, Resolution>) {
        let (width, height) = (resolution.width.max(1), resolution.height.max(1));
        if (self.surface_config.width, self.surface_config.height) != (width, height) {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.is_surface_refreshed = true;
        }
    }

    #[run_after_previous_and(component(Target))]
    fn prepare_texture(&mut self, target: &mut Target, device: Single<'_, GpuDevice>) {
        if self.is_surface_refreshed {
            self.current_texture = None;
            self.surface.configure(&device.device, &self.surface_config);
            self.is_surface_refreshed = false;
        }
        let texture = self
            .surface
            .get_current_texture()
            .expect("internal error: cannot retrieve surface texture");
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        target.set_surface(view);
        self.current_texture = Some(texture);
    }

    pub(crate) fn invalidate_surface(&mut self) {
        self.is_surface_invalid = true;
    }

    pub(crate) fn present_texture(&mut self) {
        self.current_texture
            .take()
            .expect("internal error: surface texture not initialized")
            .present();
    }

    fn is_immediate_mode_supported(&self) -> bool {
        self.surface
            .get_supported_present_modes(&self.adapter)
            .contains(&PresentMode::Immediate)
    }

    #[allow(unsafe_code)]
    fn create_surface(instance: &Instance, window: &winit::window::Window) -> Surface {
        unsafe { instance.create_surface(&window) }
    }

    fn retrieve_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        executor::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }))
        .expect("no supported graphic adapter found")
    }

    fn create_surface_config(
        surface: &Surface,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) -> SurfaceConfiguration {
        let formats = surface.get_supported_formats(adapter);
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: if formats.is_empty() {
                panic!("internal error: surface is incompatible with adapter")
            } else {
                formats[0]
            },
            width: if width == 0 { 1 } else { width },
            height: if height == 0 { 1 } else { height },
            present_mode: PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(adapter)[0],
        }
    }
}

struct Window {
    handle: Arc<RwLock<winit::window::Window>>,
    properties: Option<WindowProperties>,
}

#[singleton]
impl Window {
    const DEFAULT_PROPERTIES: WindowProperties = WindowProperties {
        title: String::new(),
        is_cursor_visible: true,
    };

    fn build(handle: Arc<RwLock<winit::window::Window>>) -> impl Built<Self> {
        EntityBuilder::new(Self {
            handle,
            properties: None,
        })
    }

    #[run]
    fn update(
        &mut self,
        title: Option<Single<'_, WindowTitle>>,
        cursor_visibility: Option<Single<'_, CursorVisibility>>,
    ) {
        let new_properties = WindowProperties {
            title: title
                .as_ref()
                .map_or(&Self::DEFAULT_PROPERTIES.title, |t| &t.0)
                .clone(),
            is_cursor_visible: cursor_visibility
                .map_or(Self::DEFAULT_PROPERTIES.is_cursor_visible, |t| t.is_visible),
        };
        if Some(&new_properties) != self.properties.as_ref() {
            let window = self
                .handle
                .write()
                .expect("internal error: not writable window");
            window.set_cursor_visible(new_properties.is_cursor_visible);
            window.set_title(&new_properties.title);
            self.properties = Some(new_properties);
        }
    }

    fn handle(&self) -> RwLockReadGuard<'_, winit::window::Window> {
        self.handle
            .read()
            .expect("internal error: not readable window")
    }
}

#[derive(PartialEq, Eq)]
struct WindowProperties {
    title: String,
    is_cursor_visible: bool,
}
