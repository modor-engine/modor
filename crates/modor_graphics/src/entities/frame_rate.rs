/// A frame rate limit.
///
/// The limit is only applied if the [`runner`](crate::runner()) is used.
///
/// If no frame rate limit is defined, then the default frame rate is
/// [`FrameRate::VSync`](FrameRate::VSync).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_physics::*;
/// #
/// # fn no_run() {
/// let mut app = App::new()
///      .with_entity(GraphicsModule::build(
///          WindowSettings::default()
///              .size(SurfaceSize::new(800, 600))
///              .title("title"),
///      ))
///     .with_entity(FrameRateLimit::from(FrameRate::FPS(60)))
///     .run(modor_graphics::runner);
/// # }
/// ```
#[derive(SingletonComponent, NoSystem)]
pub struct FrameRateLimit {
    frame_rate: FrameRate,
}

impl From<FrameRate> for FrameRateLimit {
    fn from(frame_rate: FrameRate) -> Self {
        info!("frame rate limit initialized to {:?}", frame_rate);
        Self { frame_rate }
    }
}

impl FrameRateLimit {
    /// Get the frame rate limit.
    pub fn get(&self) -> FrameRate {
        self.frame_rate
    }

    /// Set the frame rate limit.
    pub fn set(&mut self, frame_rate: FrameRate) {
        self.frame_rate = frame_rate;
        info!("frame rate limit set to {:?}", frame_rate);
    }
}

/// A frame rate.
///
/// On some platforms like web and mobile, the frame rate might be limited in any case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameRate {
    /// A limit in frames per second.
    ///
    /// `FrameRate::FPS(0)` is equivalent to `FrameRate::Unlimited`.
    FPS(u16),
    /// A limit based on vertical synchronization.
    ///
    /// This is the most optimal mode for mobile and web.
    VSync,
    /// No limitation.
    Unlimited,
}
