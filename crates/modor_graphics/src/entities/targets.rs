use crate::{Camera2D, RenderTarget, Size, Texture, TextureBuffer, Window};
use modor::{BuiltEntity, EntityBuilder};
use modor_resources::{IndexResKey, ResKey};

/// The key of the [`RenderTarget`] created by [`window_target`].
pub const WINDOW_TARGET: ResKey<RenderTarget> = ResKey::new("window(modor_graphics)");
/// The key of the [`Camera2D`] created by [`window_target`].
pub const WINDOW_CAMERA_2D: ResKey<Camera2D> = ResKey::new("window(modor_graphics)");
/// The keys of the [`RenderTarget`] created by [`texture_target`].
pub const TEXTURE_TARGETS: IndexResKey<RenderTarget> = IndexResKey::new("texture(modor_graphics)");
/// The keys of the [`Camera2D`] created by [`texture_target`].
pub const TEXTURE_CAMERAS_2D: IndexResKey<Camera2D> = IndexResKey::new("texture(modor_graphics)");
/// The keys of the [`Texture`] created by [`texture_target`].
pub const TARGET_TEXTURES: IndexResKey<Texture> = IndexResKey::new("target(modor_graphics)");

/// Creates a window target entity.
///
/// The created entity contains the following components:
/// - [`RenderTarget`] with [`WINDOW_TARGET`] key
/// - [`Window`]
/// - [`Camera2D`] with [`WINDOW_CAMERA_2D`] key
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
    EntityBuilder::new()
        .component(RenderTarget::new(WINDOW_TARGET))
        .component(Window::default())
        .component(Camera2D::new(WINDOW_CAMERA_2D, WINDOW_TARGET))
}

/// Creates a texture target entity.
///
/// The created entity contains the following components:
/// - [`RenderTarget`] with [`TEXTURE_TARGETS`]`.get(index)` key
/// - [`Texture`] with [`TARGET_TEXTURES`]`.get(index)` key
/// - [`Camera2D`] with [`TEXTURE_CAMERAS_2D`]`.get(index)` key
/// - Optional [`TextureBuffer`] if `is_buffer_enabled` is `true`
///
/// Texture targets are uniquely identified by its `index`.
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
///     .with_entity(texture_target(0, Size::new(800, 600), false))
///     .run(modor_graphics::runner);
/// # }
/// ```
pub fn texture_target(index: usize, size: Size, is_buffer_enabled: bool) -> impl BuiltEntity {
    let target_key = TEXTURE_TARGETS.get(index);
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Texture::from_size(TARGET_TEXTURES.get(index), size))
        .component(Camera2D::new(TEXTURE_CAMERAS_2D.get(index), target_key))
        .component_option(is_buffer_enabled.then(TextureBuffer::default))
}
