use crate::components::pipeline::Pipeline2D;
use crate::Dynamics2D;
use modor::SingleMut;
use modor_math::Vec2;

/// The positioning of a 2D entity.
///
/// # Related components
///
/// - [`Dynamics2D`]
/// - [`Collider2D`]
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// App::new()
///     .with_entity(modor_physics::module())
///     .with_entity(object());
///
/// fn object() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .with(|t| t.position = Vec2::new(0.25, -0.25))
///         .with(|t| t.size = Vec2::ONE * 0.2)
/// }
/// ```
#[derive(Component, Debug)]
pub struct Transform2D {
    /// Position of the entity in world units.
    pub position: Vec2,
    /// Size of the entity in world units.
    pub size: Vec2,
    /// Rotation of the entity in radians.
    pub rotation: f32,
}

#[systems]
impl Transform2D {
    /// Creates a new transform.
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
        }
    }

    #[run_after(component(Dynamics2D), component(Pipeline2D))]
    fn update(&mut self, dynamics: &Dynamics2D, mut pipeline: SingleMut<'_, '_, Pipeline2D>) {
        let pipeline = pipeline.get_mut();
        if let Some(body) = dynamics.handle.and_then(|handle| pipeline.body_mut(handle)) {
            self.position.x = body.translation().x;
            self.position.y = body.translation().y;
            self.rotation = body.rotation().angle();
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Transform2D {
    fn clone(&self) -> Self {
        Self {
            position: self.position,
            size: self.size,
            rotation: self.rotation,
        }
    }
}
