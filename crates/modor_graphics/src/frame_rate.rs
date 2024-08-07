use crate::platform;
use instant::Instant;
use std::time::Duration;
use wgpu::PresentMode;

/// A frame rate limit.
///
/// # Examples
///
/// See [`Window`](crate::Window).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FrameRate {
    /// A limit based on vertical synchronization.
    ///
    /// This is the most optimal mode for mobile and web.
    VSync,
    /// A limit in frames per second.
    ///
    /// `FrameRate::Fps(0)` is equivalent to `FrameRate::Unlimited`.
    Fps(u16),
    /// No limitation.
    Unlimited,
}

impl FrameRate {
    pub(crate) fn present_mode(self, has_immediate_mode: bool) -> PresentMode {
        if matches!(self, Self::VSync) || !has_immediate_mode {
            PresentMode::Fifo
        } else {
            PresentMode::Immediate
        }
    }

    pub(crate) fn sleep(self, start: Instant) {
        if let Self::Fps(frames_per_second) = self {
            if frames_per_second > 0 {
                let frame_time = Duration::from_secs_f64(1. / f64::from(frames_per_second));
                Self::sleep_internal(start, frame_time);
            }
        }
    }

    fn sleep_internal(start: Instant, frame_time: Duration) {
        let update_time = Instant::now().duration_since(start);
        if let Some(remaining_time) = frame_time.checked_sub(update_time) {
            platform::sleep(remaining_time);
        }
    }
}

#[cfg(test)]
mod utils_tests {
    use crate::FrameRate;
    use instant::{Duration, Instant};
    use wgpu::PresentMode;

    #[test]
    fn retrieve_present_mode() {
        assert_eq!(
            FrameRate::Unlimited.present_mode(true),
            PresentMode::Immediate
        );
        assert_eq!(
            FrameRate::Fps(60).present_mode(true),
            PresentMode::Immediate
        );
        assert_eq!(FrameRate::VSync.present_mode(true), PresentMode::Fifo);
        assert_eq!(FrameRate::Unlimited.present_mode(false), PresentMode::Fifo);
        assert_eq!(FrameRate::Fps(60).present_mode(false), PresentMode::Fifo);
        assert_eq!(FrameRate::VSync.present_mode(false), PresentMode::Fifo);
    }

    #[test]
    fn run_with_frame_rate() {
        modor_internal::retry!(10, assert_duration(FrameRate::Unlimited, 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::VSync, 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(0), 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(1), 500, 1000, 1200));
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(5), 100, 200, 300));
    }

    fn assert_duration(
        frame_rate: FrameRate,
        external_sleep_millis: u64,
        min_millis: u64,
        max_millis: u64,
    ) {
        let update_start = Instant::now();
        spin_sleep::sleep(Duration::from_millis(external_sleep_millis));
        frame_rate.sleep(update_start);
        let update_end = Instant::now();
        assert!(update_end.duration_since(update_start) >= Duration::from_millis(min_millis));
        assert!(update_end.duration_since(update_start) <= Duration::from_millis(max_millis));
    }
}
