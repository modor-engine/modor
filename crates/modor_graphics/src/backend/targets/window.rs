use crate::backend::targets::{CreatedTarget, Target};
use futures::executor;
use wgpu::{
    Adapter, Backends, CommandEncoder, Device, Instance, PowerPreference, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat,
    TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::window::Window;

// coverage: off (window cannot be tested)

pub(crate) struct WindowTarget {
    immediate_mode_supported: bool,
    instance: Instance,
    adapter: Adapter,
    surface: Surface,
    surface_config: SurfaceConfiguration,
    current_texture: Option<SurfaceTexture>,
}

impl WindowTarget {
    pub(crate) fn new(window: &Window) -> CreatedTarget<Self> {
        let instance =
            Instance::new(wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all));
        let surface = Self::create_surface(&instance, window);
        let adapter = Self::retrieve_adapter(&instance, &surface);
        let (device, queue) = super::retrieve_device(&adapter);
        let surface_config = Self::create_surface_config(window, &surface, &adapter);
        surface.configure(&device, &surface_config);
        CreatedTarget {
            target: Self {
                immediate_mode_supported: surface
                    .get_supported_present_modes(&adapter)
                    .contains(&PresentMode::Immediate),
                instance,
                adapter,
                surface,
                surface_config,
                current_texture: None,
            },
            device,
            queue,
        }
    }

    fn retrieve_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        executor::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }))
        .expect("no supported graphic adapter found")
    }

    #[allow(unsafe_code)]
    fn create_surface(instance: &Instance, window: &Window) -> Surface {
        unsafe { instance.create_surface(&window) }
    }

    fn create_surface_config(
        window: &Window,
        surface: &Surface,
        adapter: &Adapter,
    ) -> SurfaceConfiguration {
        let window_size = window.inner_size();
        let (width, height) = (window_size.width, window_size.height);
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

impl Target for WindowTarget {
    fn size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    fn texture_format(&self) -> TextureFormat {
        self.surface_config.format
    }

    fn retrieve_buffer(&self, _device: &Device) -> Vec<u8> {
        panic!("internal error: surface buffer cannot be retrieved")
    }

    fn set_size(&mut self, width: u32, height: u32, device: &Device) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(device, &self.surface_config);
    }

    fn toggle_vsync(&mut self, enabled: bool, device: &Device) {
        let previous_mode = self.surface_config.present_mode;
        self.surface_config.present_mode = if enabled || !self.immediate_mode_supported {
            PresentMode::Fifo
        } else {
            PresentMode::Immediate
        };
        if previous_mode != self.surface_config.present_mode {
            self.surface.configure(device, &self.surface_config);
        }
    }

    fn prepare_texture(&mut self) -> TextureView {
        let texture = self
            .surface
            .get_current_texture()
            .expect("internal error: cannot retrieve surface texture");
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        self.current_texture = Some(texture);
        view
    }

    fn render(&mut self, queue: &Queue, encoder: CommandEncoder) {
        queue.submit(std::iter::once(encoder.finish()));
        self.current_texture
            .take()
            .expect("internal error: no surface texture to render")
            .present();
    }

    fn refresh_surface(&mut self, window: &Window, device: &Device) {
        self.surface = Self::create_surface(&self.instance, window);
        self.surface_config = Self::create_surface_config(window, &self.surface, &self.adapter);
        self.surface.configure(device, &self.surface_config);
        self.current_texture = None;
    }
}
