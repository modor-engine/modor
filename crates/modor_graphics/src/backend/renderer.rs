use std::mem;
use wgpu::{
    Adapter, Backends, Buffer, Device, DeviceDescriptor, Extent3d, Features, Instance, Limits,
    MapMode, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTexture, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};
use winit::window::Window;

pub(super) const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;
pub(super) const TARGET_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub(crate) struct Renderer {
    target: RenderTarget,
    target_size: (u32, u32),
    device: Device,
    queue: Queue,
    depth_buffer: TextureView,
}

impl Renderer {
    pub(crate) fn for_surface(window: &Window) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = Self::retrieve_window_adapter(&instance, &surface);
        let (device, queue) = Self::retrieve_device(&adapter);
        let window_size = window.inner_size();
        let target_size = (window_size.width, window_size.height);
        let surface_config = Self::create_surface_config(target_size, &surface, &adapter);
        surface.configure(&device, &surface_config);
        let depth_buffer = Self::create_depth_buffer(&device, target_size);
        Self {
            target: RenderTarget::Surface {
                surface,
                surface_config,
            },
            target_size,
            device,
            queue,
            depth_buffer,
        }
    }

    pub(crate) fn for_texture(target_size: (u32, u32)) -> Self {
        let instance = Instance::new(Backends::all());
        let adapter = Self::retrieve_default_adapter(&instance);
        let (device, queue) = Self::retrieve_device(&adapter);
        let depth_buffer = Self::create_depth_buffer(&device, target_size);
        let texture = Self::create_target_texture(&device, target_size);
        let bytes_per_pixel = mem::size_of::<u32>() as u64;
        let unpadded_bytes_per_row = u64::from(target_size.0) * bytes_per_pixel;
        let align = u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        let buffer = Self::create_output_buffer(&device, target_size, padded_bytes_per_row);
        Self {
            target: RenderTarget::Texture {
                texture,
                buffer,
                unpadded_bytes_per_row,
                padded_bytes_per_row,
            },
            target_size,
            device,
            queue,
            depth_buffer,
        }
    }

    pub(crate) fn target_view(&mut self) -> TargetView<'_> {
        match &self.target {
            RenderTarget::Surface { .. } => {
                panic!("internal error: surface target cannot be accessed")
            }
            RenderTarget::Texture {
                buffer,
                unpadded_bytes_per_row,
                padded_bytes_per_row,
                ..
            } => {
                let buffer_slice = buffer.slice(..);
                let buffer_future = buffer_slice.map_async(MapMode::Read);
                self.device.poll(wgpu::Maintain::Wait);
                pollster::block_on(buffer_future)
                    .expect("internal error: cannot retrieve target view buffer");
                TargetView {
                    buffer,
                    unpadded_bytes_per_row: *unpadded_bytes_per_row as usize,
                    padded_bytes_per_row: *padded_bytes_per_row as usize,
                }
            }
        }
    }

    pub(crate) fn target_size(&self) -> (u32, u32) {
        self.target_size
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 && (width, height) != self.target_size {
            self.target_size = (width, height);
            self.depth_buffer = Self::create_depth_buffer(&self.device, self.target_size);
            match &mut self.target {
                RenderTarget::Surface {
                    surface,
                    surface_config,
                } => {
                    surface_config.width = width;
                    surface_config.height = height;
                    surface.configure(&self.device, &surface_config);
                }
                RenderTarget::Texture { .. } => {
                    todo!("resize texture")
                }
            }
        }
    }

    pub(super) fn depth_buffer(&self) -> &TextureView {
        &self.depth_buffer
    }

    pub(super) fn target(&self) -> &RenderTarget {
        &self.target
    }

    pub(super) fn device(&self) -> &Device {
        &self.device
    }

    pub(super) fn queue(&self) -> &Queue {
        &self.queue
    }

    fn retrieve_window_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        }))
        .expect("no supported graphic adapter found")
    }

    fn retrieve_default_adapter(instance: &Instance) -> Adapter {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
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

    fn create_depth_buffer(device: &Device, target_size: (u32, u32)) -> TextureView {
        let desc = TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width: target_size.0,
                height: target_size.1,
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

    fn create_target_texture(device: &Device, target_size: (u32, u32)) -> Texture {
        let desc = TextureDescriptor {
            label: Some("modor_target_texture"),
            size: Extent3d {
                width: target_size.0,
                height: target_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TARGET_TEXTURE_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        };
        device.create_texture(&desc)
    }

    fn create_output_buffer(
        device: &Device,
        target_size: (u32, u32),
        padded_bytes_per_row: u64,
    ) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modor_target_output_buffer"),
            size: (padded_bytes_per_row * u64::from(target_size.1)),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}

pub(crate) struct TargetView<'a> {
    buffer: &'a Buffer,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl TargetView<'_> {
    pub(crate) fn unpadded_bytes_per_row(&self) -> usize {
        self.unpadded_bytes_per_row
    }

    pub(crate) fn padded_bytes_per_row(&self) -> usize {
        self.padded_bytes_per_row
    }

    pub(crate) fn use_buffer_slice<F, O>(&self, f: F) -> O
    where
        F: FnOnce(&[u8]) -> O,
    {
        f(&self.buffer.slice(..).get_mapped_range())
    }
}

impl Drop for TargetView<'_> {
    fn drop(&mut self) {
        self.buffer.unmap()
    }
}

pub(super) enum RenderTarget {
    Surface {
        surface: Surface,
        surface_config: SurfaceConfiguration,
    },
    Texture {
        texture: Texture,
        buffer: Buffer,
        unpadded_bytes_per_row: u64,
        padded_bytes_per_row: u64,
    },
}

impl RenderTarget {
    pub(super) fn format(&self) -> TextureFormat {
        match self {
            RenderTarget::Surface { surface_config, .. } => surface_config.format,
            RenderTarget::Texture { .. } => TARGET_TEXTURE_FORMAT,
        }
    }

    pub(crate) fn output(&self) -> RenderOutput {
        match self {
            RenderTarget::Surface { surface, .. } => RenderOutput::Surface(
                surface
                    .get_current_texture()
                    .expect("internal error: cannot retrieve surface texture"),
            ),
            RenderTarget::Texture {
                texture,
                buffer,
                padded_bytes_per_row,
                ..
            } => RenderOutput::Texture(texture, buffer, *padded_bytes_per_row),
        }
    }
}

pub(super) enum RenderOutput<'a> {
    Surface(SurfaceTexture),
    Texture(&'a Texture, &'a Buffer, u64),
}

impl RenderOutput<'_> {
    pub(super) fn texture(&self) -> &Texture {
        match self {
            RenderOutput::Surface(texture) => &texture.texture,
            RenderOutput::Texture(texture, _, _) => texture,
        }
    }

    pub(super) fn buffer(&self) -> Option<&Buffer> {
        match self {
            RenderOutput::Surface(_) => None,
            RenderOutput::Texture(_, buffer, _) => Some(buffer),
        }
    }

    pub(super) fn padded_bytes_per_row(&self) -> u64 {
        match self {
            RenderOutput::Surface(_) => panic!("internal error: no bytes per row for surface"),
            RenderOutput::Texture(_, _, padded_bytes_per_row) => *padded_bytes_per_row,
        }
    }

    pub(super) fn finish(self) {
        match self {
            RenderOutput::Surface(texture) => texture.present(),
            RenderOutput::Texture(_, _, _) => (),
        }
    }
}
