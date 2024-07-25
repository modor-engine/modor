use crate::gpu::Gpu;
use crate::validation;
use enum_iterator::Sequence;
use fxhash::FxHashMap;
use modor::{Node, RootNode};
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

/// An anti-aliasing mode.
///
/// # Examples
///
/// See [`Window`](crate::Window).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Sequence)]
#[non_exhaustive]
pub enum AntiAliasingMode {
    /// Anti-aliasing is disabled.
    #[default]
    None,
    /// Multi-Sample Anti-Aliasing is enabled with 2 samples.
    MsaaX2,
    /// Multi-Sample Anti-Aliasing is enabled with 4 samples.
    MsaaX4,
    /// Multi-Sample Anti-Aliasing is enabled with 8 samples.
    MsaaX8,
    /// Multi-Sample Anti-Aliasing is enabled with 16 samples.
    MsaaX16,
}

impl AntiAliasingMode {
    /// Returns the number of samples applied for anti-aliasing.
    pub const fn sample_count(self) -> u32 {
        match self {
            Self::None => 1,
            Self::MsaaX2 => 2,
            Self::MsaaX4 => 4,
            Self::MsaaX8 => 8,
            Self::MsaaX16 => 16,
        }
    }
}

#[derive(Default, RootNode, Node)]
pub(crate) struct SupportedAntiAliasingModes {
    modes: FxHashMap<TextureFormat, Vec<AntiAliasingMode>>,
}

impl SupportedAntiAliasingModes {
    pub(crate) fn get(&mut self, gpu: &Gpu, format: TextureFormat) -> &[AntiAliasingMode] {
        self.modes
            .entry(format)
            .or_insert_with(|| Self::supported_modes(gpu, format))
    }

    fn supported_modes(gpu: &Gpu, format: TextureFormat) -> Vec<AntiAliasingMode> {
        enum_iterator::all::<AntiAliasingMode>()
            .filter(|&mode| Self::is_mode_supported(gpu, format, mode))
            .collect()
    }

    fn is_mode_supported(gpu: &Gpu, format: TextureFormat, mode: AntiAliasingMode) -> bool {
        if mode == AntiAliasingMode::None {
            return true;
        }
        validation::validate_wgpu(gpu, true, || {
            gpu.device.create_texture(&TextureDescriptor {
                label: Some("modor_texture:msaa_check"),
                size: Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: mode.sample_count(),
                dimension: TextureDimension::D2,
                format,
                usage: TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        })
        .is_ok()
    }
}
