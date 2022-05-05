use crate::FrameRate;
use std::time::{Duration, Instant};

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
                spin_sleep::sleep(remaining_time);
            }
        }
    }
}

#[cfg(test)]
mod utils_tests {
    use crate::FrameRate;
    use approx::assert_abs_diff_eq;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn calculate_nearest_multiple() {
        assert_eq!(super::nearest_multiple(0, 4), 0);
        assert_eq!(super::nearest_multiple(1, 4), 4);
        assert_eq!(super::nearest_multiple(4, 4), 4);
        assert_eq!(super::nearest_multiple(5, 4), 8);
    }

    #[test]
    fn normalize() {
        assert_abs_diff_eq!(super::normalize(-1., -1., 1., 1., 2.), 1.);
        assert_abs_diff_eq!(super::normalize(0., -1., 1., 1., 2.), 1.5);
        assert_abs_diff_eq!(super::normalize(1., -1., 1., 1., 2.), 2.);
        assert_abs_diff_eq!(super::normalize(0., 0., 0., 1., 2.), 1.);
    }

    #[test]
    fn run_with_frame_rate() {
        retry!(10, assert_duration(FrameRate::Unlimited, 100, 100, 150));
        retry!(10, assert_duration(FrameRate::VSync, 100, 100, 150));
        retry!(10, assert_duration(FrameRate::FPS(0), 100, 100, 150));
        retry!(10, assert_duration(FrameRate::FPS(1), 500, 1000, 1200));
        retry!(10, assert_duration(FrameRate::FPS(5), 100, 200, 300));
    }

    fn assert_duration(
        frame_rate: FrameRate,
        external_sleep_millis: u64,
        min_millis: u64,
        max_millis: u64,
    ) {
        let update_start = Instant::now();
        super::run_with_frame_rate(Instant::now(), frame_rate, || {
            thread::sleep(Duration::from_millis(external_sleep_millis))
        });
        let update_end = Instant::now();
        assert!(update_end.duration_since(update_start) >= Duration::from_millis(min_millis));
        assert!(update_end.duration_since(update_start) <= Duration::from_millis(max_millis));
    }
}
