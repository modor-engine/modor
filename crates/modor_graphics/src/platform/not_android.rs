use winit::event_loop::{EventLoop, EventLoopBuilder};

// coverage: off (cannot be run during tests)
pub(crate) fn event_loop() -> EventLoop<()> {
    EventLoopBuilder::new()
        .build()
        .expect("graphics runner initialization has failed")
}
// coverage: on
