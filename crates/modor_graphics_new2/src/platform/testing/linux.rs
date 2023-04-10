use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::platform::unix::EventLoopBuilderExtUnix;

/// The context of a [test runner](fn@crate::test_runner).
///
/// Should be created only once during the whole test suite execution.
///
/// # Platform-specific
///
/// The test runner is supported only on Window and Linux. On other platforms, the runner
/// does nothing.
#[doc(hidden)]
pub struct TestRunnerContext {
    event_loop: EventLoop<()>,
}

impl Default for TestRunnerContext {
    fn default() -> Self {
        Self {
            event_loop: winit::event_loop::EventLoopBuilder::new()
                .with_any_thread(true)
                .build(),
        }
    }
}

impl TestRunnerContext {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn event_loop(&mut self) -> Option<&mut EventLoop<()>> {
        Some(&mut self.event_loop)
    }

    pub(crate) fn run(
        event_loop: &mut EventLoop<()>,
        f: impl FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
    ) {
        event_loop.run_return(f);
    }
}
