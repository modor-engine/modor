use crate::components::renderer::Renderer;
use crate::data::size::NonZeroSize;
use crate::{GpuContext, RenderTarget, Texture};
use modor::Single;
use std::mem;
use std::num::NonZeroU32;
use wgpu::{Buffer, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, MapMode, SubmissionIndex};

#[derive(Component, Debug, Default)]
pub struct TextureBuffer {
    buffer: Option<BufferDetails>,
    data: Vec<u8>,
    renderer_version: Option<u8>,
}

#[systems]
impl TextureBuffer {
    #[run_after(component(Texture), component(Renderer), component(RenderTarget))]
    fn update(&mut self, texture: Option<&Texture>, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        let texture_size = texture.and_then(Texture::size).map(NonZeroSize::from);
        if state.is_removed() || texture_size.is_none() {
            self.buffer = None;
            self.data = vec![];
        }
        if let (Some(context), Some(texture_size), Some(texture)) =
            (state.context(), texture_size, texture)
        {
            if self.is_update_needed(texture_size) {
                self.buffer = Some(BufferDetails {
                    buffer: Self::create_buffer(context, texture_size),
                    size: texture_size,
                });
            }
            let buffer = self
                .buffer
                .as_ref()
                .expect("internal error: texture buffer not created");
            let index = Self::copy_texture_in_buffer(context, texture, buffer);
            self.data = Self::retrieve_buffer(context, buffer, index);
        }
    }

    pub fn get(&self) -> &[u8] {
        &self.data
    }

    fn is_update_needed(&self, texture_size: NonZeroSize) -> bool {
        self.buffer
            .as_ref()
            .map_or(true, |buffer| buffer.size != texture_size)
    }

    fn create_buffer(context: &GpuContext, size: NonZeroSize) -> Buffer {
        let padded_bytes_per_row = Self::calculate_padded_row_bytes(size.width.into());
        context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modor_target_output_buffer"),
            size: u64::from(padded_bytes_per_row * u32::from(size.height)),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn copy_texture_in_buffer(
        context: &GpuContext,
        texture: &Texture,
        buffer: &BufferDetails,
    ) -> SubmissionIndex {
        let padded_row_bytes = Self::calculate_padded_row_bytes(buffer.size.width.into());
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_texture_buffer_encoder"),
        };
        let mut encoder = context.device.create_command_encoder(&descriptor);
        encoder.copy_texture_to_buffer(
            texture.inner().texture.as_image_copy(),
            ImageCopyBuffer {
                buffer: &buffer.buffer,
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
                width: buffer.size.width.into(),
                height: buffer.size.height.into(),
                depth_or_array_layers: 1,
            },
        );
        context.queue.submit(Some(encoder.finish()))
    }

    fn retrieve_buffer(
        context: &GpuContext,
        buffer: &BufferDetails,
        index: SubmissionIndex,
    ) -> Vec<u8> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(buffer.size.width.into());
        let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(buffer.size.width.into());
        let slice = buffer.buffer.slice(..);
        slice.map_async(MapMode::Read, |_| ());
        context
            .device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(index));
        let data = slice
            .get_mapped_range()
            .chunks(padded_row_bytes as usize)
            .flat_map(|a| &a[..unpadded_row_bytes as usize])
            .copied()
            .collect();
        buffer.buffer.unmap();
        data
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
}

#[derive(Debug)]
struct BufferDetails {
    buffer: Buffer,
    size: NonZeroSize,
}
