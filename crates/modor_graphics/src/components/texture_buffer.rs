use crate::components::renderer::Renderer;
use crate::data::size::NonZeroSize;
use crate::{GpuContext, RenderTarget, Size, Texture};
use modor::SingleRef;
use std::mem;
use std::num::NonZeroU32;
use wgpu::{Buffer, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, MapMode, SubmissionIndex};

/// The content of a GPU buffer texture.
///
/// This component retrieves the content of a texture at each update.
/// Note that retrieving GPU data may have a significant impact on performance.
///
/// # Requirements
///
/// The component retrieves texture content only if:
/// - graphics [`module`](crate::module()) is initialized
/// - [`Texture`](Texture) component is in the same entity
///
/// # Related components
///
/// - [`Texture`](Texture)
///
/// # Entity functions creating this component
///
/// - [`texture_target`](crate::texture_target())
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// fn screenshot() -> impl BuiltEntity {
///     let target_key = ResKey::unique("main");
///     let texture_key = ResKey::unique("main");
///     EntityBuilder::new()
///         .component(RenderTarget::new(target_key))
///         .component(Texture::from_size(texture_key, Size::new(800, 600)))
///         .component(TextureBuffer::default())
///         .component(Screenshot)
/// }
///
/// #[derive(Component)]
/// struct Screenshot;
///
/// #[systems]
/// impl Screenshot {
///     #[run]
///     fn save(buffer: &TextureBuffer) {
///         let rgba_data = buffer.get();
///         // save screenshot on disk...
///     }
/// }
/// ```
#[derive(Component, Debug, Default)]
pub struct TextureBuffer {
    buffer: Option<BufferDetails>,
    data: Vec<u8>,
    renderer_version: Option<u8>,
}

#[systems]
impl TextureBuffer {
    #[run_after(component(Texture), component(Renderer), component(RenderTarget))]
    fn update(&mut self, texture: Option<&Texture>, renderer: Option<SingleRef<'_, '_, Renderer>>) {
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

    /// Returns the RGBA texture buffer.
    ///
    /// Returns empty buffer if there is no associated [`Texture`](Texture) component
    /// or if the buffer is not yet synchronized.
    pub fn get(&self) -> &[u8] {
        &self.data
    }

    /// Returns the texture size.
    ///
    /// Returns [`Size::ZERO`](Size::ZERO) if there is no associated [`Texture`](Texture) component
    /// or if the buffer is not yet synchronized.
    pub fn size(&self) -> Size {
        self.buffer.as_ref().map_or(Size::ZERO, |b| b.size.into())
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
                            .expect("internal error: cannot render empty buffer")
                            .into(),
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
