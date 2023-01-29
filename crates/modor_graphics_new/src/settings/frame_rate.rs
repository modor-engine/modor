use modor::{Built, EntityBuilder};

pub struct FrameRate {
    pub(crate) limit: FrameRateLimit,
}

#[singleton]
impl FrameRate {
    /// Builds with a `fps` limit in frames per second.
    ///
    /// `fps` equal to zero means there is no limit.
    pub fn build_with_limit(fps: u16) -> impl Built<Self> {
        EntityBuilder::new(Self {
            limit: FrameRateLimit::Fps(fps),
        })
    }

    /// Builds with a limit based on vertical synchronization.
    ///
    /// This is the most optimal mode for mobile and web.
    pub fn build_with_vsync() -> impl Built<Self> {
        EntityBuilder::new(Self {
            limit: FrameRateLimit::VSync,
        })
    }

    /// Builds without limitation.
    pub fn build_unlimited() -> impl Built<Self> {
        EntityBuilder::new(Self {
            limit: FrameRateLimit::Unlimited,
        })
    }

    pub fn set_with_limit(&mut self, fps: u16) {
        self.limit = FrameRateLimit::Fps(fps);
    }

    pub fn set_with_vsync(&mut self) {
        self.limit = FrameRateLimit::VSync;
    }

    pub fn set_unlimited(&mut self) {
        self.limit = FrameRateLimit::Unlimited;
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum FrameRateLimit {
    Fps(u16),
    VSync,
    Unlimited,
}
