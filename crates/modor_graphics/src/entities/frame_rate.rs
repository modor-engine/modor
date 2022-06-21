use modor::{Built, EntityBuilder};

/// A frame rate limit.
///
/// The limit is only applied if the [`runner`](crate::runner()) is used.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Default if missing**: `FrameRateLimit::build(FrameRate::Unlimited)`
///
/// # Examples
///
/// ```rust
/// # use modor::App;
/// # use modor_graphics::{FrameRate, FrameRateLimit, GraphicsModule, SurfaceSize, WindowSettings};
/// # use modor_physics::PhysicsModule;
/// #
/// # fn no_run() {
/// let mut app = App::new()
///      .with_entity(GraphicsModule::build(
///          WindowSettings::default()
///              .size(SurfaceSize::new(800, 600))
///              .title("title"),
///      ))
///     .with_entity(FrameRateLimit::build(FrameRate::FPS(60)))
///     .run(modor_graphics::runner);
/// # }
/// ```
pub struct FrameRateLimit {
    frame_rate: FrameRate,
}

#[singleton]
impl FrameRateLimit {
    /// Builds the entity.
    pub fn build(frame_rate: FrameRate) -> impl Built<Self> {
        EntityBuilder::new(Self { frame_rate })
    }

    /// Get the frame rate limit.
    #[must_use]
    pub fn get(&self) -> FrameRate {
        self.frame_rate
    }

    /// Set the frame rate limit.
    pub fn set(&mut self, frame_rate: FrameRate) {
        self.frame_rate = frame_rate;
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
