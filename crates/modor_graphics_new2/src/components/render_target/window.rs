use crate::components::render_target::core::TargetCore;
use crate::data::size::NonZeroSize;
use crate::{Color, FrameRate, Renderer, Window};
use wgpu::{
    PresentMode, RenderPass, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};
use winit::window::WindowId;

#[derive(Debug)]
pub(crate) struct WindowTarget {
    core: TargetCore,
    handle_id: WindowId,
    surface: Surface,
    surface_config: SurfaceConfiguration,
    current_texture: Option<SurfaceTexture>,
    has_immediate_mode: bool,
}

impl WindowTarget {
    pub(crate) fn new(window: &Window, renderer: &mut Renderer) -> Option<Self> {
        let surface = window.create_surface(&renderer.instance)?;
        let format = Self::texture_format(&surface, renderer);
        let size = window.surface_size();
        let surface_config = Self::create_surface_config(&surface, format, size, renderer);
        surface.configure(&renderer.device, &surface_config);
        Some(Self {
            core: TargetCore::new(size, renderer),
            handle_id: window.handle_id(),
            has_immediate_mode: Self::has_immediate_mode(&surface, renderer),
            surface,
            surface_config,
            current_texture: None,
        })
    }

    pub(crate) fn handle_id(&self) -> WindowId {
        self.handle_id
    }

    pub(crate) fn core(&self) -> &TargetCore {
        &self.core
    }

    pub(crate) fn updated(
        mut self,
        window: &mut Window,
        renderer: &Renderer,
        frame_rate: FrameRate,
    ) -> Option<Self> {
        let size = window.surface_size();
        let has_surface_config_changed = self.update_surface_config(frame_rate, size);
        let was_surface_invalid = self.recreate_surface_if_invalidated(window, renderer)?;
        if has_surface_config_changed || was_surface_invalid {
            self.current_texture = None;
            self.surface
                .configure(&renderer.device, &self.surface_config);
            self.has_immediate_mode = Self::has_immediate_mode(&self.surface, renderer);
        }
        self.core.update(size, renderer);
        Some(self)
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        background_color: Color,
        renderer: &Renderer,
    ) -> RenderPass<'_> {
        let texture = self.current_texture.insert(
            self.surface
                .get_current_texture()
                .expect("internal error: cannot retrieve surface texture"),
        );
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        self.core
            .begin_render_pass(background_color, renderer, view)
    }

    pub(crate) fn end_render_pass(&mut self, renderer: &Renderer) {
        self.core.submit_command_queue(None, None, renderer);
        self.current_texture
            .take()
            .expect("internal error: surface texture not initialized")
            .present();
    }

    fn texture_format(surface: &Surface, renderer: &mut Renderer) -> TextureFormat {
        *renderer
            .window_texture_format
            .insert(renderer.window_texture_format.unwrap_or_else(|| {
                surface
                    .get_supported_formats(&renderer.adapter)
                    .into_iter()
                    .next()
                    .expect("internal error: surface is incompatible with adapter")
            }))
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

    fn recreate_surface_if_invalidated(
        &mut self,
        window: &mut Window,
        renderer: &Renderer,
    ) -> Option<bool> {
        Some(if window.is_surface_invalid {
            self.surface = window.create_surface(&renderer.instance)?;
            window.is_surface_invalid = false;
            true
        } else {
            false
        })
    }

    fn create_surface_config(
        surface: &Surface,
        format: TextureFormat,
        size: NonZeroSize,
        renderer: &Renderer,
    ) -> SurfaceConfiguration {
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.into(),
            height: size.height.into(),
            present_mode: PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(&renderer.adapter)[0],
        }
    }

    fn has_immediate_mode(surface: &Surface, renderer: &Renderer) -> bool {
        surface
            .get_supported_present_modes(&renderer.adapter)
            .contains(&PresentMode::Immediate)
    }
}
