use crate::backend::targets::{CreatedTarget, Target};
use std::mem;
use std::num::NonZeroU32;
use wgpu::{
    Adapter, Backends, Buffer, CommandEncoder, Device, Extent3d, ImageCopyBuffer, Instance,
    MapMode, Queue, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

pub(crate) struct TextureTarget {
    target_size: (u32, u32),
    texture: Texture,
    buffer: Buffer,
    unpadded_row_bytes: u64,
    padded_row_bytes: u64,
}

impl TextureTarget {
    pub(crate) fn new(width: u32, height: u32) -> CreatedTarget<Self> {
        let instance = Instance::new(Backends::all());
        let adapter = Self::retrieve_adapter(&instance);
        let (device, queue) = super::retrieve_device(&adapter);
        let texture = Self::create_texture(&device, width, height);
        let unpadded_bytes_per_row = Self::calculate_unpadded_row_bytes(width);
        let padded_bytes_per_row = Self::calculate_padded_row_bytes(unpadded_bytes_per_row);
        let buffer = Self::create_buffer(&device, height, padded_bytes_per_row);
        CreatedTarget {
            target: Self {
                target_size: (width, height),
                texture,
                buffer,
                unpadded_row_bytes: unpadded_bytes_per_row,
                padded_row_bytes: padded_bytes_per_row,
            },
            device,
            queue,
        }
    }

    fn retrieve_adapter(instance: &Instance) -> Adapter {
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("no supported graphic adapter found")
    }

    fn create_texture(device: &Device, width: u32, height: u32) -> Texture {
        let desc = TextureDescriptor {
            label: Some("modor_target_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TEXTURE_FORMAT,
            usage: no_mutation!(TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC),
        };
        device.create_texture(&desc)
    }

    fn create_buffer(device: &Device, height: u32, padded_bytes_per_row: u64) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modor_target_output_buffer"),
            size: (padded_bytes_per_row * u64::from(height)),
            usage: no_mutation!(wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
            mapped_at_creation: false,
        })
    }

    fn calculate_unpadded_row_bytes(width: u32) -> u64 {
        let bytes_per_pixel = mem::size_of::<u32>() as u64;
        u64::from(width) * bytes_per_pixel
    }

    fn calculate_padded_row_bytes(unpadded_bytes_per_row: u64) -> u64 {
        let align = u64::from(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padded_bytes_per_row_padding
    }
}
impl Target for TextureTarget {
    fn size(&self) -> (u32, u32) {
        self.target_size
    }

    fn texture_format(&self) -> TextureFormat {
        TEXTURE_FORMAT
    }

    fn retrieve_buffer(&self, device: &Device) -> Vec<u8> {
        let slice = self.buffer.slice(..);
        let _ = slice.map_async(MapMode::Read);
        device.poll(wgpu::Maintain::Wait);
        let content = slice
            .get_mapped_range()
            .chunks(self.padded_row_bytes as usize)
            .map(|a| &a[..self.unpadded_row_bytes as usize])
            .flatten()
            .copied()
            .collect();
        self.buffer.unmap();
        content
    }

    fn set_size(&mut self, width: u32, height: u32, device: &Device) {
        self.target_size = (width, height);
        self.texture = Self::create_texture(device, width, height);
        self.unpadded_row_bytes = Self::calculate_unpadded_row_bytes(width);
        self.padded_row_bytes = Self::calculate_padded_row_bytes(self.unpadded_row_bytes);
        self.buffer = Self::create_buffer(&device, height, self.padded_row_bytes);
    }

    fn prepare_texture(&mut self) -> TextureView {
        self.texture.create_view(&TextureViewDescriptor::default())
    }

    fn render(&mut self, queue: &Queue, mut encoder: CommandEncoder) {
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        NonZeroU32::new(self.padded_row_bytes as u32)
                            .expect("internal error: cannot render empty buffer"),
                    ),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: self.target_size.0,
                height: self.target_size.1,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(std::iter::once(encoder.finish()));
    }
}
