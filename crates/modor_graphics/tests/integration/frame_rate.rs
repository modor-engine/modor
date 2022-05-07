use modor::testing::TestApp;
use modor::{entity, App, Built, EntityBuilder, SingleMut};
use modor_graphics::{FrameRate, FrameRateLimit};

struct FrameRateLimitUpdater(FrameRate);

#[entity]
impl FrameRateLimitUpdater {
    fn build(new_frame_rate: FrameRate) -> impl Built<Self> {
        EntityBuilder::new(Self(new_frame_rate))
    }

    #[run]
    fn update(&self, mut frame_rate: SingleMut<'_, FrameRateLimit>) {
        frame_rate.set(self.0);
    }
}

#[test]
fn use_frame_rate_limit() {
    let mut app: TestApp = App::new()
        .with_entity(FrameRateLimit::build(FrameRate::FPS(60)))
        .with_entity(FrameRateLimitUpdater::build(FrameRate::VSync))
        .into();
    app.assert_singleton::<FrameRateLimit>()
        .has(|l: &FrameRateLimit| assert_eq!(l.get(), FrameRate::FPS(60)));
    app.update();
    app.assert_singleton::<FrameRateLimit>()
        .has(|l: &FrameRateLimit| assert_eq!(l.get(), FrameRate::VSync));
}
