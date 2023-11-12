use crate::runner::state::RunnerState;
use crate::testing::TestRunnerContext;
use modor::App;
use winit::event::{Event, WindowEvent};
use winit::window::Window as WindowHandle;

/// Runner mainly used to test with a window.
///
/// `f` is run after each `app` update.
///
/// After `update_count` updates of `app`, the runner returns. `events` parameter contains
/// all events that should be sent at the end of each update.
///
/// # Platform-specific
///
/// Only supported on Window and Linux. On other platforms, the runner does nothing.
#[doc(hidden)]
pub fn test_runner(
    app: App,
    context: &mut TestRunnerContext,
    update_count: u32,
    mut f: impl FnMut(UpdateState<'_>) -> App,
) {
    context.event_loop().map_or_else(
        || panic!("test runner only supported on windows and linux platforms"),
        |l| {
            let mut state = RunnerState::new(app, l);
            let mut update_id = 0;
            TestRunnerContext::run(l, move |event, event_loop| {
                let is_update = matches!(
                    event,
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    }
                );
                state.treat_event(event, event_loop);
                if is_update {
                    let mut next_events = Vec::new();
                    let next_events_mut = &mut next_events;
                    state.run(|a, w| {
                        let update_state = UpdateState {
                            app: a,
                            window: w,
                            update_id,
                            next_events: next_events_mut,
                        };
                        f(update_state)
                    });
                    for event in next_events {
                        state.treat_event(event, event_loop);
                    }
                    update_id += 1;
                }
                if update_count == update_id {
                    event_loop.exit();
                }
            });
        },
    );
}

#[doc(hidden)]
pub struct UpdateState<'a> {
    pub app: App,
    pub window: &'a mut WindowHandle,
    pub update_id: u32,
    pub next_events: &'a mut Vec<Event<()>>,
}
