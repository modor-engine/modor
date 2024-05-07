pub(crate) fn init_canvas(_handle: &winit::window::Window) {
    // does nothing
}

pub(crate) fn run_event_loop(
    event_loop: winit::event_loop::EventLoop<()>,
    event_handler: impl FnMut(winit::event::Event<()>, &winit::event_loop::EventLoopWindowTarget<()>),
) {
    event_loop
        .run(event_handler)
        .expect("graphics event loop failed");
}

pub(crate) fn gpu_limits() -> wgpu::Limits {
    wgpu::Limits::default()
}
