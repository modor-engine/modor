use crate::components::render_target::core::TargetCore;
use crate::data::size::NonZeroSize;
use crate::{Color, FrameRate, GpuContext, Window};
use std::sync::Arc;
use wgpu::{
    PresentMode, RenderPass, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

#[derive(Debug)]
pub(crate) struct WindowTarget {
    core: TargetCore,
    surface: Arc<Surface>,
    surface_config: SurfaceConfiguration,
    current_texture: Option<SurfaceTexture>,
    has_immediate_mode: bool,
}

impl WindowTarget {
    pub(crate) fn new(window: &Window, context: &GpuContext) -> Option<Self> {
        let surface = window.surface()?;
        let format = context.surface_texture_format?;
        let size = window.size().into();
        let surface_config = Self::create_surface_config(&surface, format, size, context);
        surface.configure(&context.device, &surface_config);
        Some(Self {
            core: TargetCore::new(size, context),
            has_immediate_mode: Self::has_immediate_mode(&surface, context),
            surface,
            surface_config,
            current_texture: None,
        })
    }

    pub(crate) fn core(&self) -> &TargetCore {
        &self.core
    }

    pub(crate) fn updated(
        mut self,
        window: &mut Window,
        context: &GpuContext,
        frame_rate: FrameRate,
    ) -> Self {
        let size = window.size().into();
        let has_surface_config_changed = self.update_surface_config(frame_rate, size);
        let is_surface_refreshed = self.recreate_surface_if_refreshed(window);
        if has_surface_config_changed || is_surface_refreshed {
            self.current_texture = None;
            self.surface
                .configure(&context.device, &self.surface_config);
            self.has_immediate_mode = Self::has_immediate_mode(&self.surface, context);
        }
        self.core.update(size, context);
        self
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        background_color: Color,
        context: &GpuContext,
    ) -> RenderPass<'_> {
        let texture = self.current_texture.insert(
            self.surface
                .get_current_texture()
                .expect("internal error: cannot retrieve surface texture"),
        );
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        self.core.begin_render_pass(background_color, context, view)
    }

    pub(crate) fn end_render_pass(&mut self, context: &GpuContext) {
        self.core.submit_command_queue(context);
        self.current_texture
            .take()
            .expect("internal error: surface texture not initialized")
            .present();
    }

    fn update_surface_config(&mut self, frame_rate: FrameRate, size: NonZeroSize) -> bool {
        let mut config = self.surface_config.clone();
        config.width = size.width.into();
        config.height = size.height.into();
        config.present_mode = frame_rate.present_mode(self.has_immediate_mode);
        if self.surface_config == config {
            false
        } else {
            self.surface_config = config;
            true
        }
    }

    fn recreate_surface_if_refreshed(&mut self, window: &mut Window) -> bool {
        if let Some(surface) = window.refreshed_surface() {
            self.surface = surface;
            true
        } else {
            false
        }
    }

    fn create_surface_config(
        surface: &Surface,
        format: TextureFormat,
        size: NonZeroSize,
        context: &GpuContext,
    ) -> SurfaceConfiguration {
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.into(),
            height: size.height.into(),
            present_mode: PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(&context.adapter)[0],
        }
    }

    fn has_immediate_mode(surface: &Surface, context: &GpuContext) -> bool {
        surface
            .get_supported_present_modes(&context.adapter)
            .contains(&PresentMode::Immediate)
    }
}
