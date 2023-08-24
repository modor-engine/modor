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
///     .with_entity(AntiAliasing::Smaa)
///     .with_entity(window_target())
///     .run(modor_graphics::runner);
/// # }
/// ```
#[derive(SingletonComponent, NoSystem, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum AntiAliasing {
    /// Anti-aliasing is disabled.
    #[default]
    None,
    /// Anti-aliasing is enabled, and [SMAA](https://github.com/iryoku/smaa) technique is used.
    Smaa(u32),
}

impl AntiAliasing {
    pub(crate) fn smaa_sample_count(self) -> u32 {
        match self {
            Self::None => 1,
            Self::Smaa(sample_count) => sample_count,
        }
    }
}
