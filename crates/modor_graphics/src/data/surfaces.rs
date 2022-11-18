/// The size in pixels of a surface in which rendering is done.
///
/// # Examples
///
/// See [`GraphicsModule`](crate::GraphicsModule).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SurfaceSize {
    /// Surface width in pixels.
    pub width: u32,
    /// Surface height in pixels.
    pub height: u32,
}

impl SurfaceSize {
    /// Creates a new size.
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
