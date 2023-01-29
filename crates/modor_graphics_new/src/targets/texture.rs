use crate::settings::rendering::{Resolution, DEFAULT_TARGET_HEIGHT, DEFAULT_TARGET_WIDTH};
use crate::targets::GpuDevice;
use crate::targets::Target;
use futures::executor;
use modor::{Built, EntityBuilder, Single};
use std::mem;
use std::num::NonZeroU32;
use wgpu::{
    Adapter, Backends, Buffer, Device, Extent3d, ImageCopyBuffer, Instance, MapMode,
    RequestAdapterOptions, Texture, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

pub(crate) struct TextureTarget {
    size: (u32, u32),
    texture: Texture,
    buffer: Buffer,
}

#[singleton]
impl TextureTarget {
    const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

    pub(crate) fn build() -> impl Built<Self> {
        let (width, height) = (DEFAULT_TARGET_WIDTH, DEFAULT_TARGET_HEIGHT);
        let instance =
            Instance::new(wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all));
        let adapter = Self::retrieve_adapter(&instance);
        let (device, queue) = super::retrieve_device_and_queue(&adapter);
        let texture = Self::create_texture(&device, width, height);
        let buffer = Self::create_buffer(&device, width, height);
        EntityBuilder::new(Self {
            size: (width, height),
            texture,
            buffer,
        })
        .inherit_from(Target::build(device, queue, width, height, Self::FORMAT))
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
            self.texture = Self::create_texture(&device.device, width, height);
            self.buffer = Self::create_buffer(&device.device, width, height);
        }
    }

    #[run_after_previous_and(entity(Target))]
    fn prepare_texture(&self, target: &mut Target) {
        let surface = self.texture.create_view(&TextureViewDescriptor::default());
        target.set_surface(surface);
    }

    pub(crate) fn copy_texture_to_buffer(&self, target: &mut Target) {
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size.0);
        target.encoder_mut().copy_texture_to_buffer(
            self.texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &self.buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        NonZeroU32::new(padded_row_bytes)
                            .expect("internal error: cannot render empty buffer"),
                    ),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: self.size.0,
                height: self.size.1,
                depth_or_array_layers: 1,
            },
        );
    }

    pub(crate) fn retrieve_buffer(&self, device: &Device) -> Vec<u8> {
        let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(self.size.0);
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size.0);
        let slice = self.buffer.slice(..);
        slice.map_async(MapMode::Read, |_| ());
        device.poll(wgpu::Maintain::Wait);
        let content = slice
            .get_mapped_range()
            .chunks(padded_row_bytes as usize)
            .flat_map(|a| &a[..unpadded_row_bytes as usize])
            .copied()
            .collect();
        self.buffer.unmap();
        content
    }

    fn retrieve_adapter(instance: &Instance) -> Adapter {
        executor::block_on(instance.request_adapter(&RequestAdapterOptions::default()))
            .expect("no supported graphic adapter found")
    }

    fn create_texture(device: &Device, width: u32, height: u32) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("modor_target_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        })
    }

    fn calculate_padded_row_bytes(width: u32) -> u32 {
        let unpadded_bytes_per_row = Self::calculate_unpadded_row_bytes(width);
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padded_bytes_per_row_padding
    }

    #[allow(clippy::cast_possible_truncation)]
    fn calculate_unpadded_row_bytes(width: u32) -> u32 {
        let bytes_per_pixel = mem::size_of::<u32>() as u32;
        width * bytes_per_pixel
    }

    fn create_buffer(device: &Device, width: u32, height: u32) -> Buffer {
        let padded_bytes_per_row = Self::calculate_padded_row_bytes(width);
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modor_target_output_buffer"),
            size: u64::from(padded_bytes_per_row * height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
