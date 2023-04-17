use crate::runner::state::RunnerState;
use crate::testing::TestRunnerContext;
use modor::App;
use winit::event::Event;
use winit::event_loop::ControlFlow;
use winit::window::Window as WindowHandle;

/// Runner mainly used to test windows.
///
/// # Platform-specific
///
/// Only supported on Window and Linux. On other platforms, the runner does nothing.
#[doc(hidden)]
pub fn test_runner(
    app: App,
    context: &mut TestRunnerContext,
    update_count: u32,
    mut f: impl FnMut(App, &mut WindowHandle, u32) -> App,
) {
    context.event_loop().map_or_else(
        || panic!("test runner only supported on windows and linux platforms"),
        |l| {
            let mut state = RunnerState::new(app, l);
            let mut update_id = 0;
            TestRunnerContext::run(l, move |event, _event_loop, control_flow| {
                let is_update = matches!(event, Event::MainEventsCleared);
                state.treat_event(event, control_flow);
                if is_update {
                    state.run(|a, w| f(a, w, update_id));
                    update_id += 1;
                }
                if update_count == update_id {
                    *control_flow = ControlFlow::Exit;
                }
            });
        },
    );
}
