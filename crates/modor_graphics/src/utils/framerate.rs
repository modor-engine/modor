use crate::FrameRate;
use instant::Instant;
use std::time::Duration;

#[allow(unused_variables)]
pub(crate) fn run_with_frame_rate<F>(start: Instant, frame_rate: FrameRate, f: F)
where
    F: FnOnce(),
{
    f();
    if let FrameRate::FPS(frames_per_second) = frame_rate {
        if frames_per_second > 0 {
            let update_time = Duration::from_secs_f32(1. / f32::from(frames_per_second));
            let current_update_time = Instant::now().duration_since(start);
            if let Some(remaining_time) = update_time.checked_sub(current_update_time) {
                #[cfg(not(target_arch = "wasm32"))]
                spin_sleep::sleep(remaining_time);
            }
        }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod utils_tests {
    use crate::FrameRate;
    use instant::{Duration, Instant};

    #[test]
    fn run_with_frame_rate() {
        modor_internal::retry!(10, assert_duration(FrameRate::Unlimited, 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::VSync, 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::FPS(0), 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::FPS(1), 500, 1000, 1200));
        modor_internal::retry!(10, assert_duration(FrameRate::FPS(5), 100, 200, 300));
    }

    fn assert_duration(
        frame_rate: FrameRate,
        external_sleep_millis: u64,
        min_millis: u64,
        max_millis: u64,
    ) {
        let update_start = Instant::now();
        super::run_with_frame_rate(Instant::now(), frame_rate, || {
            spin_sleep::sleep(Duration::from_millis(external_sleep_millis));
        });
        let update_end = Instant::now();
        assert!(update_end.duration_since(update_start) >= Duration::from_millis(min_millis));
        assert!(update_end.duration_since(update_start) <= Duration::from_millis(max_millis));
    }
}
