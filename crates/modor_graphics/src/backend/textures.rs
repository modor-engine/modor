use crate::backend::renderer::Renderer;
use image::RgbaImage;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Extent3d,
    FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Sampler, SamplerDescriptor,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};

pub(crate) struct Texture {
    bind_group: BindGroup,
    is_transparent: bool,
}

impl Texture {
    pub(crate) fn new(
        image: Image,
        mag_linear: bool,
        is_repeated: bool,
        label_suffix: &str,
        renderer: &Renderer,
    ) -> Self {
        let dimensions = image.data.dimensions();
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = Self::create_texture(label_suffix, size, renderer);
        Self::write_texture(image.data, size, &texture, renderer);
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = Self::create_sampler(mag_linear, is_repeated, label_suffix, renderer);
        let bind_group = Self::create_bind_group(&view, &sampler, label_suffix, renderer);
        Self {
            bind_group,
            is_transparent: image.is_transparent,
        }
    }

    pub(crate) fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub(super) fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn create_texture(label_suffix: &str, size: Extent3d, renderer: &Renderer) -> wgpu::Texture {
        renderer.device().create_texture(&TextureDescriptor {
            label: Some(&format!("modor_texture_{label_suffix}")),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        })
    }

    fn write_texture(
        rgba: RgbaImage,
        size: Extent3d,
        texture: &wgpu::Texture,
        renderer: &Renderer,
    ) {
        renderer.queue().write_texture(
            ImageCopyTexture {
                aspect: TextureAspect::All,
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
            },
            &rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * size.width),
                rows_per_image: std::num::NonZeroU32::new(size.height),
            },
            size,
        );
    }

    fn create_sampler(
        mag_linear: bool,
        is_repeated: bool,
        label_suffix: &str,
        renderer: &Renderer,
    ) -> Sampler {
        renderer.device().create_sampler(&SamplerDescriptor {
            label: Some(&format!("modor_texture_sampler_{label_suffix}")),
            address_mode_u: if is_repeated {
                AddressMode::Repeat
            } else {
                AddressMode::ClampToEdge
            },
            address_mode_v: if is_repeated {
                AddressMode::Repeat
            } else {
                AddressMode::ClampToEdge
            },
            address_mode_w: if is_repeated {
                AddressMode::Repeat
            } else {
                AddressMode::ClampToEdge
            },
            min_filter: FilterMode::Nearest,
            mag_filter: if mag_linear {
                FilterMode::Linear
            } else {
                FilterMode::Nearest
            },
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: f32::MAX,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        })
    }

    fn create_bind_group(
        view: &TextureView,
        sampler: &Sampler,
        label_suffix: &str,
        renderer: &Renderer,
    ) -> BindGroup {
        renderer.device().create_bind_group(&BindGroupDescriptor {
            layout: renderer.texture_bind_group_layout(),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
            label: Some(&format!("modor_texture_bind_group_{label_suffix}")),
        })
    }
}

#[derive(Debug)]
pub(crate) struct Image {
    pub(crate) data: RgbaImage,
    pub(crate) is_transparent: bool,
}
