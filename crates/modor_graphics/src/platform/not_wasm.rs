// coverage: off (window cannot be tested)

pub(crate) fn init_canvas(_handle: &winit::window::Window) {
    // does nothing
}

pub(crate) fn run_event_loop(
    event_loop: winit::event_loop::EventLoop<()>,
    mut app: impl winit::application::ApplicationHandler + 'static,
) {
    event_loop
        .run_app(&mut app)
        .expect("graphics event loop failed");
}

pub(crate) fn update_canvas_cursor(_handle: &winit::window::Window, _is_cursor_show: bool) {
    // does nothing
}

pub(crate) fn surface_size(
    _handle: &winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
) -> winit::dpi::PhysicalSize<u32> {
    size
}

// coverage: on

pub(crate) fn gpu_limits() -> wgpu::Limits {
    wgpu::Limits::default()
}

pub(crate) fn sleep(duration: std::time::Duration) {
    spin_sleep::sleep(duration);
    log::trace!("slept for {}ns", duration.as_nanos());
}
