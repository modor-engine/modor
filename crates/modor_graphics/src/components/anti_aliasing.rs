/// The anti-aliasing configuration.
///
/// Anti-aliasing is disabled by default ([`AntiAliasing::None`](AntiAliasing::None)).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics::module())
///     .with_entity(AntiAliasing::MsaaX2)
///     .with_entity(window_target())
///     .run(modor_graphics::runner);
/// # }
/// ```
#[derive(SingletonComponent, NoSystem, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum AntiAliasing {
    /// Anti-aliasing is disabled.
    #[default]
    None,
    /// Multi-Sample Anti-Aliasing is enabled with 2 samples.
    MsaaX2,
    /// Multi-Sample Anti-Aliasing is enabled with 4 samples.
    MsaaX4,
    /// Multi-Sample Anti-Aliasing is enabled with 8 samples.
    MsaaX8,
}

impl AntiAliasing {
    /// Returns the number of samples applied for anti-aliasing.
    pub fn sample_count(self) -> u32 {
        match self {
            Self::None => 1,
            Self::MsaaX2 => 2,
            Self::MsaaX4 => 4,
            Self::MsaaX8 => 8,
        }
    }
}
