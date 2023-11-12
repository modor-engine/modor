use winit::event_loop::{EventLoop, EventLoopBuilder};

pub(crate) fn event_loop() -> EventLoop<()> {
    EventLoopBuilder::new()
        .build()
        .expect("graphics runner initialization has failed")
}
