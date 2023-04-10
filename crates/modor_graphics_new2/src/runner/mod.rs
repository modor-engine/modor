use modor::App;
use state::RunnerState;
use winit::event_loop::EventLoop;

pub fn runner(app: App) {
    let event_loop = EventLoop::new();
    let mut state = RunnerState::new(app, &event_loop);
    event_loop.run(move |event, _event_loop, control_flow| {
        state.treat_event(event, control_flow);
    });
}

pub(crate) mod app;
pub(crate) mod testing;

mod display;
mod state;
