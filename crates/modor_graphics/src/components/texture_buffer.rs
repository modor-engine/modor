use crate::components::renderer::Renderer;
use crate::data::size::NonZeroSize;
use crate::{Color, GpuContext, RenderTarget, Size, Texture};
use fxhash::FxHashMap;
use modor::{ComponentSystems, SingleRef};
use modor_input::{Fingers, Gamepads, Keyboard, Mouse};
use std::num::NonZeroU32;
use wgpu::{
    Buffer, BufferView, CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, MapMode,
    SubmissionIndex,
};

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
///         .with(|b| b.part = TextureBufferPart::All)
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
    /// Part of the texture that can be accessed.
    ///
    /// Default is [`TextureBufferPart::All`].
    pub part: TextureBufferPart,
    buffer: Option<BufferDetails>,
    data: Vec<u8>,
    pixels: FxHashMap<Pixel, Color>,
    renderer_version: Option<u8>,
}

#[systems]
impl TextureBuffer {
    const COMPONENT_COUNT_PER_PIXEL: u32 = 4;

    #[run_as(action(PreTextureBufferPartUpdate))]
    fn reset_part(&mut self) {
        if let TextureBufferPart::Pixels(pixels) = &mut self.part {
            pixels.clear();
        }
    }

    #[run_after(
        component(Texture),
        component(Renderer),
        component(RenderTarget),
        action(TextureBufferPartUpdate)
    )]
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
            let view = Self::buffer_view(context, buffer, index);
            match &self.part {
                TextureBufferPart::All => self.data = Self::retrieve_buffer(&view, buffer),
                TextureBufferPart::Pixels(pixels) => {
                    self.pixels.clear();
                    for &pixel in pixels {
                        if let Some(color) = Self::retrieve_pixel_color(pixel, &view, buffer) {
                            self.pixels.insert(pixel, color);
                        }
                    }
                }
            }
            drop(view);
            buffer.buffer.unmap();
        }
    }

    /// Returns the RGBA texture buffer.
    ///
    /// Returns empty buffer in the following cases:
    /// - There is no associated [`Texture`](Texture) component.
    /// - The buffer is not yet synchronized.
    /// - [`TextureBuffer::part`](#structfield.part) is not equal to [`TextureBufferPart::All`].
    ///
    /// Note that the read texture uses sRGB format, so further transformations might be required to obtain RGB colors.
    pub fn get(&self) -> &[u8] {
        &self.data
    }

    /// Returns a pixel color.
    ///
    /// The color is returned only if the pixel coordinates are not out of bound.
    ///
    /// In case [`TextureBuffer::part`](#structfield.part) is equal to [`TextureBufferPart::Pixels`], the pixel must
    /// be specified.
    ///
    /// Note that the read texture uses sRGB format, so further transformations might be required to obtain RGB colors.
    pub fn pixel(&self, pixel: Pixel) -> Option<Color> {
        let Some(buffer) = &self.buffer else {
            return None;
        };
        if pixel.x >= u32::from(buffer.size.width) || pixel.y >= u32::from(buffer.size.height) {
            return None;
        }
        self.pixels.get(&pixel).copied().or_else(|| {
            let color_start = Self::COMPONENT_COUNT_PER_PIXEL
                * (pixel.y * u32::from(buffer.size.width) + pixel.x);
            Self::extract_color(&self.data, color_start)
        })
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

    fn buffer_view<'a>(
        context: &GpuContext,
        buffer: &'a BufferDetails,
        index: SubmissionIndex,
    ) -> BufferView<'a> {
        let slice = buffer.buffer.slice(..);
        slice.map_async(MapMode::Read, |_| ());
        context
            .device
            .poll(wgpu::Maintain::WaitForSubmissionIndex(index));
        slice.get_mapped_range()
    }

    fn retrieve_buffer(view: &BufferView<'_>, buffer: &BufferDetails) -> Vec<u8> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(buffer.size.width.into());
        let unpadded_row_bytes = u32::from(buffer.size.width) * Self::COMPONENT_COUNT_PER_PIXEL;
        let data = view
            .chunks(padded_row_bytes as usize)
            .flat_map(|a| &a[..unpadded_row_bytes as usize])
            .copied()
            .collect();
        data
    }

    fn retrieve_pixel_color(
        pixel: Pixel,
        view: &BufferView<'_>,
        buffer: &BufferDetails,
    ) -> Option<Color> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(buffer.size.width.into());
        let color_start = pixel.y * padded_row_bytes + Self::COMPONENT_COUNT_PER_PIXEL * pixel.x;
        Self::extract_color(view, color_start)
    }

    fn extract_color(data: &[u8], start_index: u32) -> Option<Color> {
        if start_index as usize >= data.len() {
            return None;
        }
        let end_index = start_index + Self::COMPONENT_COUNT_PER_PIXEL;
        let color = &data[start_index as usize..end_index as usize];
        Some(Color::rgba(
            f32::from(color[0]) / 255.,
            f32::from(color[1]) / 255.,
            f32::from(color[2]) / 255.,
            f32::from(color[3]) / 255.,
        ))
    }

    fn calculate_padded_row_bytes(width: u32) -> u32 {
        let unpadded_bytes_per_row = width * Self::COMPONENT_COUNT_PER_PIXEL;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        unpadded_bytes_per_row + padded_bytes_per_row_padding
    }
}

#[derive(Action)]
struct PreTextureBufferPartUpdate;

// TODO: add doc + doc example + mention texture part reset
#[derive(Action)]
pub struct TextureBufferPartUpdate(
    PreTextureBufferPartUpdate,
    <Keyboard as ComponentSystems>::Action,
    <Mouse as ComponentSystems>::Action,
    <Fingers as ComponentSystems>::Action,
    <Gamepads as ComponentSystems>::Action,
);

/// The part of a texture that is retrieved from GPU.
///
/// # Examples
///
/// See [`TextureBuffer`].
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum TextureBufferPart {
    /// All data of the texture are retrieved.
    ///
    /// Note that this may have impact on performance.
    #[default]
    All,
    // TODO: mention texture part reset
    /// Only specific pixels are retrieved.
    ///
    /// In case full texture data are not needed, this should be faster than [`TextureBufferPart::All`].
    Pixels(Vec<Pixel>),
}

/// The coordinates of a pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pixel {
    /// X coordinate from left side of the texture.
    pub x: u32,
    /// Y coordinate from top side of the texture.
    pub y: u32,
}

impl Pixel {
    /// Creates new pixel coordinates.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug)]
struct BufferDetails {
    buffer: Buffer,
    size: NonZeroSize,
}
