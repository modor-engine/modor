use crate::FrameRate;
use instant::Instant;
use std::time::Duration;

#[allow(clippy::cast_precision_loss)]
pub(crate) fn world_scale(window_size: (u32, u32)) -> (f32, f32) {
    (
        f32::min(window_size.1 as f32 / window_size.0 as f32, 1.),
        f32::min(window_size.0 as f32 / window_size.1 as f32, 1.),
    )
}

pub(crate) fn nearest_multiple(value: u64, multiple: u64) -> u64 {
    let align_mask = multiple - 1;
    (value + align_mask) & !align_mask
}

pub(crate) fn normalize(
    value: f32,
    min: f32,
    max: f32,
    normalized_min: f32,
    normalized_max: f32,
) -> f32 {
    if max > min {
        ((value - min) / (max - min)).mul_add(normalized_max - normalized_min, normalized_min)
    } else {
        normalized_min
    }
}

#[allow(unused_variables)]
pub(crate) fn run_with_frame_rate<F>(start: Instant, frame_rate: FrameRate, f: F)
where
    F: FnOnce(),
{
    f();
    {
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
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod utils_tests {
    use crate::FrameRate;
    use std::time::{Duration, Instant};

    #[test]
    fn calculate_world_scale() {
        assert_approx_eq!(super::world_scale((800, 600)).0, 0.75);
        assert_approx_eq!(super::world_scale((800, 600)).1, 1.);
        assert_approx_eq!(super::world_scale((600, 800)).0, 1.);
        assert_approx_eq!(super::world_scale((600, 800)).1, 0.75);
        assert_approx_eq!(super::world_scale((800, 800)).0, 1.);
        assert_approx_eq!(super::world_scale((800, 800)).1, 1.);
    }

    #[test]
    fn calculate_nearest_multiple() {
        assert_eq!(super::nearest_multiple(0, 4), 0);
        assert_eq!(super::nearest_multiple(1, 4), 4);
        assert_eq!(super::nearest_multiple(4, 4), 4);
        assert_eq!(super::nearest_multiple(5, 4), 8);
    }

    #[test]
    fn normalize() {
        assert_approx_eq!(super::normalize(-1., -1., 1., 1., 2.), 1.);
        assert_approx_eq!(super::normalize(0., -1., 1., 1., 2.), 1.5);
        assert_approx_eq!(super::normalize(1., -1., 1., 1., 2.), 2.);
        assert_approx_eq!(super::normalize(0., 0., 0., 1., 2.), 1.);
    }

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
