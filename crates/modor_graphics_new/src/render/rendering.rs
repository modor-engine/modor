use std::any::Any;
use std::marker::PhantomData;
use wgpu::{
    BindGroupLayout, Color, CommandEncoder, CommandEncoderDescriptor, Device, Extent3d, Queue,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub struct Rendering {
    pub width: u32,
    pub height: u32,
    pub background_color: Color,
}

impl Default for Rendering {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            background_color: Color::BLACK,
        }
    }
}

#[component]
impl Rendering {
    #[must_use]
    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[must_use]
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }
}

pub(crate) struct Target<T> {
    device: Device,
    queue: Queue,
    width: u32,
    height: u32,
    depth_buffer_view: TextureView,
    camera_bind_group_layout: BindGroupLayout,
    encoder: Option<CommandEncoder>,
    surface: Option<TextureView>,
    phantom: PhantomData<T>,
}

#[component]
impl<T> Target<T>
where
    T: Any + Sync + Send,
{
    pub(crate) const CAMERA_BINDING: u32 = 0;

    fn new(device: Device, queue: Queue, width: u32, height: u32, format: TextureFormat) -> Self {
        let camera_bind_group_layout =
            Self::create_bind_group_layout(Self::CAMERA_BINDING, "camera", &device);
        Self {
            depth_buffer_view: Self::create_depth_buffer_view(&device, width, height),
            device,
            queue,
            width,
            height,
            camera_bind_group_layout,
            encoder: None,
            surface: None,
            phantom: PhantomData,
        }
    }

    #[run_after(component(Rendering))]
    fn update(&mut self, rendering: &Rendering) {
        let (width, height) = (rendering.width.max(1), rendering.height.max(1));
        if (self.width, self.height) != (width, height) {
            (self.width, self.height) = (width, height);
            self.depth_buffer_view = Self::create_depth_buffer_view(&self.device, width, height);
        }
    }

    #[run]
    fn init_command_encoder(&mut self) {
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_render_encoder"),
        };
        self.encoder = Some(self.device.create_command_encoder(&descriptor));
    }

    fn create_bind_group_layout(
        binding: u32,
        label_suffix: &str,
        device: &Device,
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some(&format!("modor_uniform_bind_group_layout_{}", label_suffix)),
        })
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
