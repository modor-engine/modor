use crate::gpu::{Gpu, GpuManager};
use crate::texture::internal::TextureLoaded;
use crate::{Color, Size, Texture};
use modor::{App, FromApp, Global, StateHandle};
use std::mem;
use std::num::NonZeroU32;
use wgpu::{
    AddressMode, Buffer, BufferView, CommandEncoderDescriptor, Extent3d, FilterMode,
    ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, MapMode, Origin3d, Sampler,
    SamplerDescriptor, SubmissionIndex, TextureAspect, TextureDescriptor, TextureDimension,
    TextureUsages, TextureView, TextureViewDescriptor,
};

// TODO: merge with glob

/// The global data of a [`Texture`](Texture).
#[derive(Debug, Global)]
pub struct TextureGlob {
    /// The size of the texture in pixels.
    pub size: Size,
    pub(crate) is_transparent: bool,
    pub(crate) view: TextureView,
    pub(crate) sampler: Sampler,
    pub(super) texture: wgpu::Texture,
    buffer: Option<Buffer>,
    submission_index: Option<SubmissionIndex>,
    is_smooth: bool,
    is_repeated: bool,
    gpu_manager: StateHandle<GpuManager>,
}

impl FromApp for TextureGlob {
    fn from_app(app: &mut App) -> Self {
        let loaded = TextureLoaded::default();
        Self::new(
            app,
            &loaded,
            Texture::DEFAULT_IS_REPEATED,
            Texture::DEFAULT_IS_SMOOTH,
            Texture::DEFAULT_IS_BUFFER_ENABLED,
        )
    }
}

impl TextureGlob {
    const COMPONENT_COUNT_PER_PIXEL: u32 = 4;

    /// Retrieves the texture buffer from the GPU.
    ///
    /// Each item is the component value of a pixel, and each pixel has 4 components (RGBA format).
    ///
    /// The returned buffer contains data only if:
    /// - The [`Texture`] buffer is enabled.
    /// - The [`Texture`] buffer has been updated.
    ///
    /// Note that retrieving data from the GPU may have a significant impact on performance.
    pub fn buffer(&self, app: &App) -> Vec<u8> {
        let gpu = self
            .gpu_manager
            .get(app)
            .get()
            .expect("internal error: not initialized GPU");
        if let (Some(buffer), Some(submission_index)) = (&self.buffer, &self.submission_index) {
            let view = Self::buffer_view(gpu, buffer, submission_index);
            let data = self.retrieve_buffer(&view);
            drop(view);
            buffer.unmap();
            data
        } else {
            vec![]
        }
    }

    /// Retrieves a pixel color from the GPU.
    ///
    /// The color is returned only if:
    /// - The [`Texture`] buffer is enabled.
    /// - The [`Texture`] buffer has been updated.
    /// - The pixel coordinates (`x`, `y`) are not out of bound.
    ///
    /// Note that retrieving data from the GPU may have a significant impact on performance.
    pub fn color(&self, app: &App, x: u32, y: u32) -> Option<Color> {
        let gpu = self
            .gpu_manager
            .get(app)
            .get()
            .expect("internal error: not initialized GPU");
        if let (Some(buffer), Some(submission_index)) = (&self.buffer, &self.submission_index) {
            let view = Self::buffer_view(gpu, buffer, submission_index);
            let color = self.retrieve_pixel_color(x, y, &view);
            drop(view);
            buffer.unmap();
            color
        } else {
            None
        }
    }

    pub(super) fn new(
        app: &mut App,
        loaded: &TextureLoaded,
        is_repeated: bool,
        is_smooth: bool,
        is_buffer_enabled: bool,
    ) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init();
        let texture = Self::create_texture(gpu, loaded);
        Self::write_texture(gpu, loaded, &texture);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = Self::create_sampler(gpu, is_repeated, is_smooth);
        let size = Size::new(loaded.image.width(), loaded.image.height());
        Self {
            size,
            is_transparent: loaded.is_transparent,
            view,
            sampler,
            texture,
            buffer: is_buffer_enabled.then(|| Self::create_buffer(gpu, size)),
            submission_index: None,
            is_smooth,
            is_repeated,
            gpu_manager: app.handle(),
        }
    }

    pub(super) fn update(
        &mut self,
        gpu: &Gpu,
        is_repeated: bool,
        is_smooth: bool,
        is_buffer_enabled: bool,
    ) {
        if self.is_smooth != is_smooth || self.is_repeated != is_repeated {
            self.sampler = Self::create_sampler(gpu, is_repeated, is_smooth);
            self.is_smooth = is_smooth;
            self.is_repeated = is_repeated;
        }
        if self.buffer.is_none() && is_buffer_enabled {
            self.buffer = Some(Self::create_buffer(gpu, self.size));
        } else if self.buffer.is_some() && !is_buffer_enabled {
            self.buffer = None;
        }
    }

    pub(super) fn update_buffer(&mut self, gpu: &Gpu) {
        self.copy_texture_in_buffer(gpu);
    }

    fn create_texture(gpu: &Gpu, loaded: &TextureLoaded) -> wgpu::Texture {
        gpu.device.create_texture(&TextureDescriptor {
            label: Some("modor_texture"),
            size: Extent3d {
                width: loaded.image.width(),
                height: loaded.image.height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Texture::DEFAULT_FORMAT,
            usage: TextureUsages::TEXTURE_BINDING // to use the texture
                | TextureUsages::COPY_DST // to use the texture
                | TextureUsages::RENDER_ATTACHMENT // to render in the texture
                | TextureUsages::COPY_SRC, // to render in the texture
            view_formats: &[],
        })
    }

    fn write_texture(context: &Gpu, loaded: &TextureLoaded, texture: &wgpu::Texture) {
        context.queue.write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &loaded.image,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * loaded.image.width()),
                rows_per_image: Some(loaded.image.height()),
            },
            Extent3d {
                width: loaded.image.width(),
                height: loaded.image.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    fn create_sampler(gpu: &Gpu, is_repeated: bool, is_smooth: bool) -> Sampler {
        let address_mode = if is_repeated {
            AddressMode::Repeat
        } else {
            AddressMode::ClampToEdge
        };
        gpu.device.create_sampler(&SamplerDescriptor {
            label: Some("modor_texture_sampler"),
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            min_filter: FilterMode::Nearest,
            mag_filter: if is_smooth {
                FilterMode::Linear
            } else {
                FilterMode::Nearest
            },
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        })
    }

    fn create_buffer(gpu: &Gpu, size: Size) -> Buffer {
        let padded_bytes_per_row = Self::calculate_padded_row_bytes(size.width);
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modor_texture_buffer"),
            size: u64::from(padded_bytes_per_row * size.height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn copy_texture_in_buffer(&mut self, gpu: &Gpu) {
        let Some(buffer) = &self.buffer else {
            return;
        };
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size.width);
        let descriptor = CommandEncoderDescriptor {
            label: Some("modor_texture_buffer_encoder"),
        };
        let mut encoder = gpu.device.create_command_encoder(&descriptor);
        encoder.copy_texture_to_buffer(
            self.texture.as_image_copy(),
            ImageCopyBuffer {
                buffer,
                layout: ImageDataLayout {
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
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );
        self.submission_index = Some(gpu.queue.submit(Some(encoder.finish())));
    }

    fn buffer_view<'a>(
        gpu: &Gpu,
        buffer: &'a Buffer,
        submission_index: &SubmissionIndex,
    ) -> BufferView<'a> {
        let slice = buffer.slice(..);
        slice.map_async(MapMode::Read, |_| ());
        gpu.device.poll(wgpu::Maintain::WaitForSubmissionIndex(
            submission_index.clone(),
        ));
        slice.get_mapped_range()
    }

    fn retrieve_buffer(&self, view: &BufferView<'_>) -> Vec<u8> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size.width);
        let unpadded_row_bytes = Self::calculate_unpadded_row_bytes(self.size.width);
        let data = view
            .chunks(padded_row_bytes as usize)
            .flat_map(|a| &a[..unpadded_row_bytes as usize])
            .copied()
            .collect();
        data
    }

    fn retrieve_pixel_color(&self, x: u32, y: u32, view: &BufferView<'_>) -> Option<Color> {
        let padded_row_bytes = Self::calculate_padded_row_bytes(self.size.width);
        let color_start = y * padded_row_bytes + Self::COMPONENT_COUNT_PER_PIXEL * x;
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
