use crate::window::WindowInit;
use modor::App;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window as WinitWindow;

// TODO: move frame rate limitation in runner ?

#[allow(clippy::single_match, clippy::collapsible_match)]
pub fn runner(mut app: App) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut window = None;
    app.run_for_singleton::<WindowInit, _>(|i| window = Some(i.create_window(&event_loop)));
    let window = window.expect("`GraphicsModule` entity not found or created in windowless mode");
    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => read_window(&window).request_redraw(),
        Event::RedrawRequested(window_id) if window_id == read_window(&window).id() => app.update(),
        Event::WindowEvent { event, window_id } if window_id == read_window(&window).id() => {
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        }
        _ => {}
    });
}

fn read_window(window: &Arc<RwLock<WinitWindow>>) -> RwLockReadGuard<'_, WinitWindow> {
    window.read().expect("internal error: cannot read window")
}
