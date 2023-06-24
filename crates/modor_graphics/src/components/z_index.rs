/// The Z-index of a rendered 2D [`Model`](crate::Model).
///
/// It is created from a [`u16`] value, where `0` is the farthest from the camera,
/// and [`u16::MAX`] the closest to the camera.
///
/// By default, the z-index of a [`Model`](crate::Model) is `0`.
///
/// # Requirements
///
/// The component is effective only if:
/// - [`Model`](crate::Model) component is in the same entity
///
/// # Related components
///
/// - [`Model`](crate::Model)
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// fn foreground() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new().with_size(Vec2::ONE * 0.5))
///         .with(Model::rectangle(MaterialKey::Foreground).with_camera_key(CameraKey))
///         .with(ZIndex2D::from(1))
/// }
///
/// fn background() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new().with_size(Vec2::ONE))
///         .with(Model::rectangle(MaterialKey::Background).with_camera_key(CameraKey))
///         .with(ZIndex2D::from(0))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct CameraKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum MaterialKey {
///     Foreground,
///     Background,
/// }
/// ```
#[must_use]
#[derive(
    Component, NoSystem, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub struct ZIndex2D(u16);

impl ZIndex2D {
    // Returns the model depth between `0.` and `1.`.
    pub(crate) fn to_normalized_f32(self) -> f32 {
        (f32::from(self.0) + 0.5) / (f32::from(u16::MAX) + 1.)
    }
}

impl From<u16> for ZIndex2D {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl From<ZIndex2D> for u16 {
    fn from(index: ZIndex2D) -> Self {
        index.0
    }
}
