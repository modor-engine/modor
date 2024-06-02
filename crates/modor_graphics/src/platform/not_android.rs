// coverage: off (window cannot be tested)

pub(crate) fn event_loop() -> winit::event_loop::EventLoop<()> {
    winit::event_loop::EventLoopBuilder::new()
        .build()
        .expect("graphics initialization failed")
}

// coverage: on
