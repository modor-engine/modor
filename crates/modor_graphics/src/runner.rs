use crate::internal::context::Context;
use crate::internal::window::WindowInit;
use modor::App;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

// TODO: move frame rate limitation in runner ?

#[allow(clippy::single_match, clippy::collapsible_match)]
pub fn runner(mut app: App) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut window = None;
    app.run_for_singleton::<WindowInit, _>(|i| window = Some(i.create_window(&event_loop)));
    let window =
        window.expect("failed to create window because `GraphicsModule` entity is not found");
    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(window_id) if window_id == window.id() => app.update(),
        Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
            WindowEvent::Resized(physical_size) => app.run_for_singleton::<Context, _>(|c| {
                c.resize(physical_size.width, physical_size.height)
            }),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => app
                .run_for_singleton::<Context, _>(|c| {
                    c.resize(new_inner_size.width, new_inner_size.height)
                }),
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}
