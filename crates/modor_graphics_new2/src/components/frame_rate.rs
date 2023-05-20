use crate::platform;
use instant::Instant;
use std::time::Duration;
use wgpu::PresentMode;

/// A frame rate limit.
///
/// The limit is only applied if the [`runner`](crate::runner()) is used.
///
/// Default frame rate is [`FrameRate::VSync`](FrameRate::VSync).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics_new2::*;
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics_new2::module())
///     .with_entity(FrameRate::Fps(120))
///     .with_entity(window())
///     .run(modor_graphics_new2::runner);
/// # }
///
/// fn window() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Window::default())
///         .with(RenderTarget::new(TargetKey))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct TargetKey;
/// ```
#[derive(SingletonComponent, NoSystem, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum FrameRate {
    /// A limit based on vertical synchronization.
    ///
    /// This is the most optimal mode for mobile and web.
    #[default]
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

    pub(crate) fn sleep(self, start: Instant, window_frame_time: Option<Duration>) {
        if let Self::Fps(frames_per_second) = self {
            if frames_per_second > 0 {
                let frame_time = Duration::from_secs_f64(1. / f64::from(frames_per_second));
                Self::sleep_internal(start, frame_time);
            }
        } else if let (Some(window_frame_time), Self::VSync) = (window_frame_time, self) {
            // sleep to reduce input lag.
            Self::sleep_internal(start, window_frame_time);
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

    #[test]
    fn run_with_frame_rate() {
        modor_internal::retry!(
            10,
            assert_duration(FrameRate::Unlimited, 100, 100, 150, None)
        );
        modor_internal::retry!(10, assert_duration(FrameRate::VSync, 100, 100, 150, None));
        modor_internal::retry!(
            10,
            assert_duration(FrameRate::VSync, 100, 200, 300, Some(200))
        );
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(0), 100, 100, 150, None));
        modor_internal::retry!(
            10,
            assert_duration(FrameRate::Fps(1), 500, 1000, 1200, None)
        );
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(5), 100, 200, 300, None));
    }

    fn assert_duration(
        frame_rate: FrameRate,
        external_sleep_millis: u64,
        min_millis: u64,
        max_millis: u64,
        window_frame_millis: Option<u64>,
    ) {
        let update_start = Instant::now();
        spin_sleep::sleep(Duration::from_millis(external_sleep_millis));
        frame_rate.sleep(update_start, window_frame_millis.map(Duration::from_millis));
        let update_end = Instant::now();
        assert!(update_end.duration_since(update_start) >= Duration::from_millis(min_millis));
        assert!(update_end.duration_since(update_start) <= Duration::from_millis(max_millis));
    }
}
