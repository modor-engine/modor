use std::sync::Arc;
use std::time::Duration;
use wgpu::Limits;
use winit::dpi::PhysicalSize;
use winit::window::Window as WindowHandle;

pub(crate) type ThreadSafeRc<T> = Arc<T>;

pub(crate) fn init_canvas(_handle: &WindowHandle) {
    // does nothing
}

pub(crate) fn update_canvas_cursor(_handle: &WindowHandle, _is_cursor_show: bool) {
    // does nothing
}

pub(crate) fn surface_size(_handle: &WindowHandle, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
    size
}

pub(crate) fn gpu_limits() -> Limits {
    Limits::default()
}

pub(crate) fn sleep(duration: Duration) {
    spin_sleep::sleep(duration);
    trace!("slept for {}ns", duration.as_nanos());
}
