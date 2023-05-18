use winit::event::Event;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

/// The context of a [test runner](crate::test_runner()).
///
/// Should be created only once during the whole test suite execution.
///
/// # Platform-specific
///
/// The test runner is supported only on Window and Linux. On other platforms, the runner
/// does nothing.
#[doc(hidden)]
#[non_exhaustive]
#[derive(Default)]
pub struct TestRunnerContext;

impl TestRunnerContext {
    pub(crate) fn event_loop(&mut self) -> Option<&mut EventLoop<()>> {
        None
    }

    pub(crate) fn run(
        _event_loop: &mut EventLoop<()>,
        _f: impl FnMut(Event<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow),
    ) {
        // unsupported
    }
}
