use crate::data::size::NonZeroSize;
use crate::resources::render_target::core::TargetCore;
use crate::{Color, GraphicsModule, Window};
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
    pub(crate) fn new(window: &Window, module: &mut GraphicsModule) -> Option<Self> {
        let surface = window.create_surface(&module.instance)?;
        let format = Self::texture_format(&surface, module);
        let size = window.surface_size();
        let surface_config = Self::create_surface_config(&surface, format, size, module);
        surface.configure(&module.device, &surface_config);
        Some(Self {
            core: TargetCore::new(size, module),
            handle_id: window.handle_id(),
            has_immediate_mode: Self::has_immediate_mode(&surface, module),
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

    pub(crate) fn updated(mut self, window: &mut Window, module: &GraphicsModule) -> Option<Self> {
        let size = window.surface_size();
        let has_surface_config_changed = self.update_surface_config(module, size);
        let was_surface_invalid = self.recreate_surface_if_invalidated(window, module)?;
        if has_surface_config_changed || was_surface_invalid {
            self.current_texture = None;
            self.surface.configure(&module.device, &self.surface_config);
            self.has_immediate_mode = Self::has_immediate_mode(&self.surface, module);
        }
        self.core.update(size, module);
        Some(self)
    }

    pub(crate) fn begin_render_pass(
        &mut self,
        background_color: Color,
        module: &GraphicsModule,
    ) -> RenderPass<'_> {
        let texture = self.current_texture.insert(
            self.surface
                .get_current_texture()
                .expect("internal error: cannot retrieve surface texture"),
        );
        let view = texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        self.core.begin_render_pass(background_color, module, view)
    }

    pub(crate) fn end_render_pass(&mut self, module: &GraphicsModule) {
        self.core.submit_command_queue(module);
        self.current_texture
            .take()
            .expect("internal error: surface texture not initialized")
            .present();
    }

    fn texture_format(surface: &Surface, module: &mut GraphicsModule) -> TextureFormat {
        *module
            .window_texture_format
            .insert(module.window_texture_format.unwrap_or_else(|| {
                surface
                    .get_supported_formats(&module.adapter)
                    .into_iter()
                    .next()
                    .expect("internal error: surface is incompatible with adapter")
            }))
    }

    fn update_surface_config(&mut self, module: &GraphicsModule, size: NonZeroSize) -> bool {
        let mut config = self.surface_config.clone();
        config.width = size.width.into();
        config.height = size.height.into();
        config.present_mode = module.present_mode(self.has_immediate_mode);
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
        module: &GraphicsModule,
    ) -> Option<bool> {
        Some(if window.is_surface_invalid {
            self.surface = window.create_surface(&module.instance)?;
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
        module: &GraphicsModule,
    ) -> SurfaceConfiguration {
        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.into(),
            height: size.height.into(),
            present_mode: PresentMode::Fifo,
            alpha_mode: surface.get_supported_alpha_modes(&module.adapter)[0],
        }
    }

    fn has_immediate_mode(surface: &Surface, module: &GraphicsModule) -> bool {
        surface
            .get_supported_present_modes(&module.adapter)
            .contains(&PresentMode::Immediate)
    }
}
