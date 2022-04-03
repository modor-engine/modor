use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits,
    PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};
use winit::window::Window;

pub(super) const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

pub(crate) struct Renderer {
    surface_size: (u32, u32),
    surface: Surface,
    surface_config: SurfaceConfiguration,
    device: Device,
    queue: Queue,
    depth_buffer: TextureView,
}

impl Renderer {
    pub(crate) fn new(window: &Window) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = Self::retrieve_adapter(&instance, &surface);
        let (device, queue) = Self::retrieve_device(&adapter);
        let window_size = window.inner_size();
        let surface_size = (window_size.width, window_size.height);
        let surface_config = Self::create_surface_config(surface_size, &surface, &adapter);
        surface.configure(&device, &surface_config);
        let depth_buffer = Self::create_depth_buffer(&device, &surface_config);
        Self {
            surface_size,
            surface,
            surface_config,
            device,
            queue,
            depth_buffer,
        }
    }

    pub(crate) fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 && (width, height) != self.surface_size {
            self.surface_size = (width, height);
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            self.depth_buffer = Self::create_depth_buffer(&self.device, &self.surface_config);
        }
    }

    pub(super) fn depth_buffer(&self) -> &TextureView {
        &self.depth_buffer
    }

    pub(super) fn surface(&self) -> &Surface {
        &self.surface
    }

    pub(super) fn surface_config(&self) -> &SurfaceConfiguration {
        &self.surface_config
    }

    pub(super) fn device(&self) -> &Device {
        &self.device
    }

    pub(super) fn queue(&self) -> &Queue {
        &self.queue
    }

    fn retrieve_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }))
        .expect("no supported graphic adapter found")
    }

    fn retrieve_device(adapter: &Adapter) -> (Device, Queue) {
        pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                features: Features::empty(),
                limits: Limits::default(),
                label: None,
            },
            None,
        ))
        .expect("error when retrieving device")
    }

    fn create_surface_config(
        surface_size: (u32, u32),
        surface: &Surface,
        adapter: &Adapter,
    ) -> SurfaceConfiguration {
        let (width, height) = surface_size;
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(adapter).unwrap(),
            width: if width == 0 { 1 } else { width },
            height: if height == 0 { 1 } else { height },
            present_mode: PresentMode::Immediate,
        }
    }

    fn create_depth_buffer(device: &Device, surface_config: &SurfaceConfiguration) -> TextureView {
        let desc = TextureDescriptor {
            label: Some("modor_depth_buffer"),
            size: Extent3d {
                width: surface_config.width,
                height: surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        };
        let texture = device.create_texture(&desc);
        texture.create_view(&TextureViewDescriptor::default())
    }
}
