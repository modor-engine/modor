/// A component to add to entities containing [`RenderTarget`] or [`InstanceRendering2D`] to exclude from picking.
///
/// This can be used for example to limit performance impact of the picking or avoid performing picking on a
/// render target displayed in another render target.
///
/// # Requirements
///
/// The component is effective only if:
/// - picking [`module`](crate::module()) is initialized
/// - [`RenderTarget`] or [`InstanceRendering2D`] component is in the same entity
///
/// # Related components
///
/// - [`RenderTarget`](modor_graphics::RenderTarget)
/// - [`InstanceRendering2D`](modor_graphics::InstanceRendering2D)
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_picking::*;
/// #
/// App::new()
///     // picking is performed for below window target
///     .with_entity(window_target())
///     // picking is not performed for below texture render target
///     .with_entity(texture_target(0, Size::new(800, 600), false).component(NoPicking));
/// ```
#[non_exhaustive]
#[derive(Debug, Default, Component, NoSystem)]
pub struct NoPicking;
