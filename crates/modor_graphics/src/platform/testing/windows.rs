use winit::event::Event;
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget};
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
use winit::platform::windows::EventLoopBuilderExtWindows;

/// The context of a [test runner](crate::test_runner()).
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
            event_loop: EventLoopBuilder::new()
                .with_any_thread(true)
                .build()
                .expect("test runner initialization has failed"),
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
        f: impl FnMut(Event<()>, &EventLoopWindowTarget<()>),
    ) {
        event_loop
            .run_on_demand(f)
            .expect("test runner update has failed");
    }
}
