use crate::platform::ThreadSafeRc;
use crate::GpuContext;
use wgpu::{Instance, Surface};
use winit::window::Window as WindowHandle;

pub(super) struct Display {
    instance: Instance,
    pub(super) renderer: ThreadSafeRc<GpuContext>,
    pub(super) surface: ThreadSafeRc<Surface>,
}

impl Display {
    pub(super) fn new(window: &WindowHandle) -> Self {
        let instance = GpuContext::instance();
        let surface = ThreadSafeRc::new(Self::create_surface(window, &instance));
        Self {
            renderer: ThreadSafeRc::new(GpuContext::new(&instance, Some(&surface))),
            instance,
            surface,
        }
    }

    pub(super) fn refresh_surface(&mut self, window: &WindowHandle) {
        self.surface = ThreadSafeRc::new(Self::create_surface(window, &self.instance));
    }

    #[allow(unsafe_code)]
    pub(super) fn create_surface(window: &WindowHandle, instance: &Instance) -> Surface {
        unsafe {
            instance
                .create_surface(&window)
                .expect("graphics backend not supported")
        }
    }
}
