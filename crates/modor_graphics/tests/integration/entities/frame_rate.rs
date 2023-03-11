use modor::{App, With};
use modor_graphics::{FrameRate, FrameRateLimit};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_frame_rate_limit() {
    App::new()
        .with_entity(FrameRateLimit::from(FrameRate::FPS(60)))
        .with_update::<(), _>(|f: &mut FrameRateLimit| {
            f.set(FrameRate::VSync);
            assert_eq!(f.get(), FrameRate::VSync);
            f.set(FrameRate::FPS(60));
            assert_eq!(f.get(), FrameRate::FPS(60));
            f.set(FrameRate::Unlimited);
            assert_eq!(f.get(), FrameRate::Unlimited);
        })
        .assert::<With<FrameRateLimit>>(1, |e| e);
}
