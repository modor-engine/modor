use winit::platform::android::EventLoopBuilderExtAndroid;

pub(crate) fn event_loop() -> winit::event_loop::EventLoop<()> {
    let android_app = modor::ANDROID_APP
        .get()
        .cloned()
        .expect("app not correctly initialized (maybe modor::main is not used ?)");
    winit::event_loop::EventLoop::builder()
        .with_android_app(android_app)
        .build()
        .expect("graphics initialization failed")
}
