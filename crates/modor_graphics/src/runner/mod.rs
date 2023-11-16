use crate::platform;
use modor::App;
use state::RunnerState;

// coverage: off (runner cannot be tested)

/// Runs application update for each rendered frame.
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
    let event_loop = platform::event_loop();
    let mut state = RunnerState::new(app, &event_loop);
    event_loop
        .run(move |event, event_loop| {
            state.treat_event(event, event_loop);
        })
        .expect("graphics runner has failed");
}

// coverage: on

pub(crate) mod app;
pub(crate) mod testing;

mod display;
mod state;
