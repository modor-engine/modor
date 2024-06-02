// coverage: off (window cannot be tested)

pub(crate) fn init_canvas(_handle: &winit::window::Window) {
    // does nothing
}

pub(crate) fn run_event_loop<F>(event_loop: winit::event_loop::EventLoop<()>, event_handler: F)
where
    F: FnMut(winit::event::Event<()>, &winit::event_loop::EventLoopWindowTarget<()>) + 'static,
{
    event_loop
        .run(event_handler)
        .expect("graphics event loop failed");
}

// coverage: on

pub(crate) fn gpu_limits() -> wgpu::Limits {
    wgpu::Limits::default()
}
