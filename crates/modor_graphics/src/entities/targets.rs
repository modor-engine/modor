use crate::{Camera2D, RenderTarget, Window};
use modor::{BuiltEntity, EntityBuilder};
use modor_resources::ResKey;

/// The key of the [`Camera2D`] created by [`window_target`].
pub const WINDOW_CAMERA_2D: ResKey<Camera2D> = ResKey::new("window(modor_graphics)");

/// Creates a window target entity.
///
/// The created entity contains the following components:
/// - [`RenderTarget`]
/// - [`Window`]
/// - [`Camera2D`]
///
/// The [`Camera2D`] key is [`WINDOW_CAMERA_2D`].
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
///     .with_entity(window_target())
///     .run(modor_graphics::runner);
/// # }
/// ```
pub fn window_target() -> impl BuiltEntity {
    let target_key = ResKey::unique("window(modor_graphics)");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Window::default())
        .component(Camera2D::new(WINDOW_CAMERA_2D, target_key))
}
