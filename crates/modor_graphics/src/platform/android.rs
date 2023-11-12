use winit::event_loop::{EventLoop, EventLoopBuilder};
use winit::platform::android::EventLoopBuilderExtAndroid;

pub(crate) fn event_loop() -> EventLoop<()> {
    let android_app = modor::ANDROID_APP
        .get()
        .cloned()
        .expect("app not correctly initialized (maybe modor::modor_main has not been used ?)");
    EventLoopBuilder::new()
        .with_android_app(android_app)
        .build()
        .expect("graphics runner initialization has failed")
}
