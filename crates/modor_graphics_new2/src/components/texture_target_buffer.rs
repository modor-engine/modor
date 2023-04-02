use crate::components::render_target::WindowTargetUpdate;
use crate::components::renderer::Renderer;
use crate::data::size::NonZeroSize;
use crate::{GpuContext, RenderTarget, Texture};
use modor::{ComponentSystems, Single};
use std::mem;
use std::num::NonZeroU32;
use wgpu::{Buffer, CommandEncoder, Extent3d, ImageCopyBuffer, MapMode};

// TODO: generalize to any texture ?

#[derive(Component, Debug, Default)]
pub struct TextureTargetBuffer {
    size: Option<NonZeroSize>,
    buffer: Option<Buffer>,
    data: Vec<u8>,
    renderer_version: Option<u8>,
}

#[systems]
impl TextureTargetBuffer {
    pub fn buffer(&self) -> &[u8] {
        &self.data
    }

    #[run_as(TextureTargetBufferUpdate)]
    fn update_buffer(&mut self, texture: Option<&Texture>, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        let texture_size = texture.and_then(Texture::size).map(NonZeroSize::from);
        if state.is_removed() || texture_size.is_none() {
            self.buffer = None;
            self.data = vec![];
        }
        if let (Some(context), Some(texture_size)) = (state.context(), texture_size) {
            if self.size != Some(texture_size) {
                self.buffer = Some(Self::create_buffer(context, texture_size));
                self.size = Some(texture_size);
            }
        }
    }

    pub(crate) fn copy_texture_to_buffer(
        &self,
        texture: &wgpu::Texture,
        encoder: &mut CommandEncoder,
    ) {
        if let (Some(buffer), Some(size)) = (&self.buffer, self.size) {
            let padded_row_bytes = Self::calculate_padded_row_bytes(size.width.into());
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                ImageCopyBuffer {
                    buffer,
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
                    width: size.width.into(),
                    height: size.height.into(),
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    #[run_after_previous_and(component(RenderTarget), component(Renderer))]
    fn retrieve_buffer(&mut self, renderer: Single<'_, Renderer>) {
        let Some(context) = renderer.state(&mut self.renderer_version).context() else { return; };
        if let (Some(buffer), Some(size)) = (&self.buffer, self.size) {
            let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(size.width.into());
            let padded_row_bytes = Self::calculate_padded_row_bytes(size.width.into());
            let slice = buffer.slice(..);
            slice.map_async(MapMode::Read, |_| ());
            context.device.poll(wgpu::Maintain::Wait);
            self.data = slice
                .get_mapped_range()
                .chunks(padded_row_bytes as usize)
                .flat_map(|a| &a[..unpadded_row_bytes as usize])
                .copied()
                .collect();
            buffer.unmap();
        }
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

#[derive(Action)]
pub(crate) struct TextureTargetBufferUpdate(
    WindowTargetUpdate,
    <Renderer as ComponentSystems>::Action,
);
