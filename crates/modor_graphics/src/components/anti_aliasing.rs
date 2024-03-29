use crate::components::renderer::{GpuContext, Renderer};
use crate::components::shader::Shader;
use crate::errors;
use modor::SingleRef;
use wgpu::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

/// The anti-aliasing configuration.
///
/// Anti-aliasing is disabled by default.
///
/// # Requirements
///
/// The component is effective only if:
/// - graphics [`module`](crate::module()) is initialized
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics::module())
///     .with_entity(AntiAliasing::from(AntiAliasingMode::MsaaX2))
///     .with_entity(window_target())
///     .run(modor_graphics::runner);
/// # }
/// ```
#[derive(SingletonComponent, Debug)]
pub struct AntiAliasing {
    /// Anti-aliasing mode.
    pub mode: AntiAliasingMode,
    supported_modes: Vec<AntiAliasingMode>,
    renderer_version: Option<u8>,
}

#[systems]
impl AntiAliasing {
    const MSAA_MODES_SAMPLE_COUNTS: [(AntiAliasingMode, u32); 4] = [
        (AntiAliasingMode::MsaaX2, 2),
        (AntiAliasingMode::MsaaX4, 4),
        (AntiAliasingMode::MsaaX8, 8),
        (AntiAliasingMode::MsaaX16, 16),
    ];

    /// Returns all supported modes.
    ///
    /// [`AntiAliasingMode::None`](AntiAliasingMode::None) is always included.
    pub fn supported_modes(&self) -> &[AntiAliasingMode] {
        &self.supported_modes
    }

    #[run_after(component(Renderer))]
    fn update(&mut self, renderer: Option<SingleRef<'_, '_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.supported_modes = vec![AntiAliasingMode::None];
        }
        if let Some(context) = state.context() {
            if self.supported_modes.len() == 1 {
                let texture_formats = context.surface_texture_format.map_or_else(
                    || vec![Shader::TEXTURE_FORMAT],
                    |format| vec![Shader::TEXTURE_FORMAT, format],
                );
                for (mode, count) in Self::MSAA_MODES_SAMPLE_COUNTS {
                    if Self::is_msaa_sample_count_supported(context, &texture_formats, count) {
                        self.supported_modes.push(mode);
                    }
                }
            }
            if !self.supported_modes.contains(&self.mode) {
                error!("anti-aliasing mode `{:?}` is not supported", self.mode);
                self.mode = AntiAliasingMode::None;
            }
        }
    }

    fn is_msaa_sample_count_supported(
        context: &GpuContext,
        texture_formats: &[TextureFormat],
        sample_count: u32,
    ) -> bool {
        errors::validate_wgpu(context, || {
            for format in texture_formats.iter().copied() {
                context.device.create_texture(&TextureDescriptor {
                    label: Some("modor_color_texture"),
                    size: Extent3d {
                        width: 1,
                        height: 1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count,
                    dimension: TextureDimension::D2,
                    format,
                    usage: TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });
            }
        })
        .is_ok()
    }
}

impl Default for AntiAliasing {
    fn default() -> Self {
        Self::from(AntiAliasingMode::None)
    }
}

impl From<AntiAliasingMode> for AntiAliasing {
    fn from(mode: AntiAliasingMode) -> Self {
        Self {
            mode,
            supported_modes: vec![AntiAliasingMode::None],
            renderer_version: None,
        }
    }
}

/// An anti-aliasing mode.
///
/// # Examples
///
/// See [`AntiAliasing`](AntiAliasing).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
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
    pub fn sample_count(self) -> u32 {
        match self {
            Self::None => 1,
            Self::MsaaX2 => 2,
            Self::MsaaX4 => 4,
            Self::MsaaX8 => 8,
            Self::MsaaX16 => 16,
        }
    }
}
