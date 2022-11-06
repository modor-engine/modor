use crate::backend::targets::{CreatedTarget, Target};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, CommandEncoder,
    Device, Extent3d, Queue, SamplerBindingType, ShaderStages, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor,
    TextureViewDimension,
};
use winit::window::Window;

pub(super) const DEPTH_TEXTURE_FORMAT: TextureFormat = TextureFormat::Depth32Float;

pub(crate) struct Renderer {
    target: Box<dyn Target>,
    device: Device,
    queue: Queue,
    depth_buffer: TextureView,
    texture_bind_group_layout: BindGroupLayout,
}

impl Renderer {
    pub(crate) fn new(target: CreatedTarget<impl Target>) -> Self {
        let (width, height) = target.target.size();
        Self {
            depth_buffer: Self::create_depth_buffer(&target.device, width, height),
            target: Box::new(target.target),
            texture_bind_group_layout: Self::create_texture_bind_group_layout(&target.device),
            device: target.device,
            queue: target.queue,
        }
    }

    pub(crate) fn target_size(&self) -> (u32, u32) {
        self.target.size()
    }

    pub(crate) fn retrieve_buffer(&self) -> Vec<u8> {
        self.target.retrieve_buffer(&self.device)
    }

    // coverage: off (no surface refresh with capture)
    pub(crate) fn refresh_surface(&mut self, window: &Window) {
        self.target.refresh_surface(window, &self.device);
    }
    // coverage: on

    pub(crate) fn set_size(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            warn!("render target size remains unchanged as new width or height is equal to zero");
            return;
        }
        if (width, height) != self.target.size() {
            self.depth_buffer = Self::create_depth_buffer(&self.device, width, height);
            self.target.set_size(width, height, &self.device);
        }
    }

    pub(crate) fn toggle_vsync(&mut self, enabled: bool) {
        self.target.toggle_vsync(enabled, &self.device);
    }

    pub(crate) fn prepare_texture(&mut self) -> TextureView {
        self.target.prepare_texture()
    }

    pub(crate) fn render(&mut self, encoder: CommandEncoder) {
        self.target.render(&self.queue, encoder);
    }

    pub(super) fn depth_buffer(&self) -> &TextureView {
        &self.depth_buffer
    }

    pub(super) fn target(&self) -> &dyn Target {
        self.target.as_ref()
    }

    pub(super) fn device(&self) -> &Device {
        &self.device
    }

    pub(super) fn queue(&self) -> &Queue {
        &self.queue
    }

    pub(crate) fn texture_bind_group_layout(&self) -> &BindGroupLayout {
        &self.texture_bind_group_layout
    }

    fn create_depth_buffer(device: &Device, width: u32, height: u32) -> TextureView {
        let desc = TextureDescriptor {
            label: Some("modor_depth_texture"),
            size: Extent3d {
                width,
                height,
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

    fn create_texture_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("modor_texture_bind_group_layout"),
        })
    }
}
