use modor::App;
use state::RunnerState;
use winit::event_loop::EventLoop;

// coverage: off (runner cannot be tested)

/// Run application update for each rendered frame.
///
/// [`DeltaTime`](modor_physics::DeltaTime) is automatically updated.<br>
/// Frame rate is limited depending on [`FrameRate`](crate::FrameRate).
///
/// Input events are automatically sent to the [`InputModule`](modor_input::InputModule).
///
/// This runner must be used instead of a call to [`App::update`](App::update)
/// inside a loop to ensure a correct window update.
///
/// # Platform-specific
///
/// - Web: a canvas with id `modor` is automatically added to the HTML body.
/// - Android: gamepad inputs are not supported.
///
/// # Examples
///
/// See [`Window`](crate::Window).
pub fn runner(app: App) {
    let event_loop = EventLoop::new();
    let mut state = RunnerState::new(app, &event_loop);
    event_loop.run(move |event, _event_loop, control_flow| {
        state.treat_event(event, control_flow);
    });
}

// coverage: on

pub(crate) mod app;
pub(crate) mod testing;

mod display;
mod state;
