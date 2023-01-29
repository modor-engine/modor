use crate::rendering::Rendering;
use crate::settings::rendering::{
    BackgroundColor, Resolution, DEFAULT_TARGET_HEIGHT, DEFAULT_TARGET_WIDTH,
};
use crate::Color;
use futures::executor;
use modor::{Built, EntityBuilder, Single};
use wgpu::{
    Adapter, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    Features, Limits, LoadOp, Operations, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

pub(crate) struct Target {
    size: (u32, u32),
    depth_buffer_view: TextureView,
    background_color: Color,
    encoder: Option<CommandEncoder>,
    surface: Option<TextureView>,
}

#[singleton]
impl Target {
    pub(crate) fn build(
        device: Device,
        queue: Queue,
        width: u32,
        height: u32,
        format: TextureFormat,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self {
            size: (DEFAULT_TARGET_WIDTH, DEFAULT_TARGET_HEIGHT),
            depth_buffer_view: Self::create_depth_buffer_view(&device, width, height),
            background_color: Color::BLACK,
            encoder: None,
            surface: None,
        })
        .with_child(Rendering::build(format, &device))
        .with_child(GpuDevice::build(device, queue))
        .with_dependency(Resolution::build(
            DEFAULT_TARGET_WIDTH,
            DEFAULT_TARGET_HEIGHT,
        ))
    }

    #[run]
    fn update_resolution(
        &mut self,
        device: Single<'_, GpuDevice>,
        resolution: Single<'_, Resolution>,
    ) {
        let (width, height) = (resolution.width.max(1), resolution.height.max(1));
        if self.size != (width, height) {
            self.size = (width, height);
            self.depth_buffer_view = Self::create_depth_buffer_view(&device.device, width, height);
        }
    }

    #[run]
    fn update_background_color(&mut self, background_color: Option<Single<'_, BackgroundColor>>) {
        self.background_color = background_color.map_or(Color::BLACK, |c| **c);
    }

    #[run]
    fn init_command_encoder(&mut self, device: Single<'_, GpuDevice>) {
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_render_encoder"),
        };
        self.encoder = Some(device.device.create_command_encoder(&descriptor));
    }

    pub(crate) fn begin_render_pass(&mut self) -> RenderPass<'_> {
        let encoder = self
            .encoder
            .as_mut()
            .expect("internal error: encoder not initialized");
        let surface = self
            .surface
            .as_ref()
            .expect("internal error: surface not initialized");
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("modor_render_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.background_color.into()),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_buffer_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }

    pub(crate) fn encoder_mut(&mut self) -> &mut CommandEncoder {
        self.encoder
            .as_mut()
            .expect("internal error: encoder not initialized")
    }

    pub(crate) fn set_surface(&mut self, surface: TextureView) {
        self.surface = Some(surface);
    }

    pub(crate) fn submit_command_queue(&mut self, device: Single<'_, GpuDevice>) {
        device.queue.submit(std::iter::once(
            self.encoder
                .take()
                .expect("internal error: encoder not initialized")
                .finish(),
        ));
    }

    fn create_depth_buffer_view(device: &Device, width: u32, height: u32) -> TextureView {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        });
        texture.create_view(&TextureViewDescriptor::default())
    }
}

pub(crate) struct GpuDevice {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
}

#[singleton]
impl GpuDevice {
    pub(crate) fn build(device: Device, queue: Queue) -> impl Built<Self> {
        EntityBuilder::new(Self { device, queue })
    }
}

fn retrieve_device_and_queue(adapter: &Adapter) -> (Device, Queue) {
    executor::block_on(adapter.request_device(
        &DeviceDescriptor {
            features: Features::empty(),
            limits: limits(),
            label: None,
        },
        None,
    ))
    .expect("error when retrieving GPU device and queue")
}

fn limits() -> Limits {
    #[cfg(target_arch = "wasm32")]
    {
        Limits::downlevel_webgl2_defaults()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Limits::default()
    }
}

pub(crate) mod texture;
pub(crate) mod window;
