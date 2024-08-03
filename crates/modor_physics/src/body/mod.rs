use crate::body::field_doc::field_doc;
use crate::pipeline::Pipeline;
use crate::user_data::ColliderUserData;
use crate::{Collision2D, CollisionGroup};
use getset::{CopyGetters, Getters};
use modor::{App, FromApp, Glob, GlobRef, GlobUpdater, Global, StateHandle};
use modor_math::Vec2;
use rapier2d::dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodyType};
use rapier2d::geometry::{
    ActiveCollisionTypes, Collider, ColliderBuilder, ColliderHandle, SharedShape,
};
use rapier2d::na::Vector2;
use rapier2d::pipeline::ActiveHooks;
use std::marker::PhantomData;

mod field_doc;
mod updater;

/// A physical 2D body.
///
/// # Examples
///
/// ```rust
/// # use std::f32::consts::FRAC_PI_2;
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// #[derive(Default, State)]
/// struct CharacterDirection(Vec2);
///
/// #[derive(FromApp)]
/// struct Character {
///     body: Glob<Body2D>,
/// }
///
/// impl Character {
///     fn init(&mut self, app: &mut App, position: Vec2, group: &Glob<CollisionGroup>) {
///         self.body
///             .updater()
///             .position(position)
///             .size(Vec2::ONE * 0.2)
///             .rotation(FRAC_PI_2)
///             .collision_group(group.to_ref())
///             .shape(Shape2D::Circle)
///             .apply(app);
///     }
///
///     fn update(&mut self, app: &mut App) {
///         self.body
///             .updater()
///             .velocity(app.get_mut::<CharacterDirection>().0 * 0.5)
///             .apply(app);
///         for collision in self.body.get(app).collisions() {
///             println!("Character is colliding with body {}", collision.other_index);
///         }
///     }
/// }
/// ```
#[derive(Debug, GlobUpdater, CopyGetters, Getters)]
pub struct Body2D {
    pub(crate) rigid_body_handle: RigidBodyHandle,
    pub(crate) collider_handle: ColliderHandle,
    /// Collision group of the collider.<br>
    /// Note that the collisions may not be updated when only the [`size`](Body2D::size) is
    /// changed. However, it is ensured the collision is detected when updating
    /// the [`position`](Body2D::position) or the [`rotation`](Body2D::rotation).<br>
    /// Default is `None` (no collision detection is performed).
    #[updater(field, for_field = "default")]
    #[getset(get = "pub")]
    pub(crate) collision_group: Option<GlobRef<CollisionGroup>>,
    pub(crate) collisions: Vec<Collision2D>,
    pipeline: StateHandle<Pipeline>,
    #[doc = field_doc!(position)]
    #[updater(inner_type, field, for_field = "Body2D::position")]
    position: PhantomData<Vec2>,
    /// Size of the body in world units.<br>
    /// Default is [`Vec2::ONE`].
    #[updater(field, for_field = "default")]
    #[getset(get_copy = "pub")]
    size: Vec2,
    #[doc = field_doc!(rotation)]
    #[updater(inner_type, field, for_field = "Body2D::rotation")]
    rotation: PhantomData<f32>,
    #[doc = field_doc!(velocity)]
    #[updater(inner_type, field, for_field = "Body2D::velocity")]
    velocity: PhantomData<Vec2>,
    #[doc = field_doc!(angular_velocity)]
    #[updater(inner_type, field, for_field = "Body2D::angular_velocity")]
    angular_velocity: PhantomData<f32>,
    #[doc = field_doc!(force)]
    #[updater(inner_type, field, for_field = "Body2D::force")]
    force: PhantomData<Vec2>,
    #[doc = field_doc!(torque)]
    #[updater(inner_type, field, for_field = "Body2D::torque")]
    torque: PhantomData<f32>,
    /// Angular inertia of the body.<br>
    /// An angular inertia of zero is considered as infinite. In this case, torque will not have
    /// any effect (even in case of collisions).<br>
    /// Default is `0.0`.
    #[updater(field, for_field = "default")]
    #[getset(get_copy = "pub")]
    mass: f32, // stored locally so that Body2D::mass() gives immediately the new value
    /// Angular inertia of the body.<br>
    /// An angular inertia of zero is considered as infinite. In this case, torque will not have
    /// any effect (even in case of collisions).<br>
    /// Default is `0.0`.
    #[updater(field, for_field = "default")]
    #[getset(get_copy = "pub")]
    angular_inertia: f32, // stored locally so that Body2D::angular_inertia() gives immediately the new value
    #[doc = field_doc!(damping)]
    #[updater(inner_type, field, for_field = "Body2D::damping")]
    damping: PhantomData<f32>,
    #[doc = field_doc!(angular_damping)]
    #[updater(inner_type, field, for_field = "Body2D::angular_damping")]
    angular_damping: PhantomData<f32>,
    #[doc = field_doc!(dominance)]
    #[updater(inner_type, field, for_field = "Body2D::dominance")]
    dominance: PhantomData<i8>,
    #[doc = field_doc!(is_ccd_enabled)]
    #[updater(inner_type, field, for_field = "Body2D::is_ccd_enabled")]
    is_ccd_enabled: PhantomData<bool>,
    #[doc = field_doc!(shape)]
    #[updater(inner_type, field, for_field = "Body2D::shape")]
    shape: PhantomData<Shape2D>,
}

impl FromApp for Body2D {
    fn from_app(app: &mut App) -> Self {
        let pipeline = app.handle::<Pipeline>();
        let (rigid_body_handle, collider_handle) = pipeline
            .get_mut(app)
            .register_body(Self::default_rigid_body(), Self::default_collider());
        Self {
            rigid_body_handle,
            collider_handle,
            pipeline,
            collision_group: None,
            collisions: vec![],
            position: PhantomData,
            size: Self::DEFAULT_SIZE,
            rotation: PhantomData,
            velocity: PhantomData,
            angular_velocity: PhantomData,
            force: PhantomData,
            torque: PhantomData,
            mass: 0.,
            angular_inertia: 0.,
            damping: PhantomData,
            angular_damping: PhantomData,
            dominance: PhantomData,
            is_ccd_enabled: PhantomData,
            shape: PhantomData,
        }
    }
}

impl Global for Body2D {
    fn init(&mut self, app: &mut App, index: usize) {
        self.collider_mut(app).user_data = ColliderUserData::new(index, usize::MAX).into();
    }
}

impl Body2D {
    const DEFAULT_POSITION: Vec2 = Vec2::ZERO;
    const DEFAULT_SIZE: Vec2 = Vec2::ONE;

    #[doc=field_doc!(position)]
    pub fn position(&self, app: &App) -> Vec2 {
        convert_vector2(*self.rigid_body(app).translation())
    }

    #[doc=field_doc!(rotation)]
    pub fn rotation(&self, app: &App) -> f32 {
        self.rigid_body(app).rotation().angle()
    }

    #[doc=field_doc!(velocity)]
    pub fn velocity(&self, app: &App) -> Vec2 {
        convert_vector2(*self.rigid_body(app).linvel())
    }

    #[doc=field_doc!(angular_velocity)]
    pub fn angular_velocity(&self, app: &App) -> f32 {
        self.rigid_body(app).angvel()
    }

    #[doc=field_doc!(force)]
    pub fn force(&self, app: &App) -> Vec2 {
        convert_vector2(self.rigid_body(app).user_force())
    }

    #[doc=field_doc!(torque)]
    pub fn torque(&self, app: &App) -> f32 {
        self.rigid_body(app).user_torque()
    }

    #[doc=field_doc!(damping)]
    pub fn damping(&self, app: &App) -> f32 {
        self.rigid_body(app).linear_damping()
    }

    #[doc=field_doc!(angular_damping)]
    pub fn angular_damping(&self, app: &App) -> f32 {
        self.rigid_body(app).angular_damping()
    }

    #[doc=field_doc!(dominance)]
    pub fn dominance(&self, app: &App) -> i8 {
        self.rigid_body(app).dominance_group()
    }

    #[doc=field_doc!(is_ccd_enabled)]
    pub fn is_ccd_enabled(&self, app: &App) -> bool {
        self.rigid_body(app).is_ccd_enabled()
    }

    #[doc=field_doc!(shape)]
    pub fn shape(&self, app: &App) -> Shape2D {
        let shape = self.collider(app).shape();
        if shape.as_cuboid().is_some() {
            Shape2D::Rectangle
        } else if shape.as_ball().is_some() {
            Shape2D::Circle
        } else {
            unreachable!("internal error: unsupported body shape")
        }
    }

    /// Returns the detected collisions.
    pub fn collisions(&self) -> &[Collision2D] {
        &self.collisions
    }

    /// Returns the detected collisions with another body from the specific collision `group`.
    pub fn collisions_with(
        &self,
        group: &Glob<CollisionGroup>,
    ) -> impl Iterator<Item = Collision2D> + '_ {
        let group_index = group.index();
        self.collisions
            .iter()
            .copied()
            .filter(move |collision| collision.other_group_index == group_index)
    }

    /// Returns whether the body collides with a body inside `group`.
    pub fn is_colliding_with(&self, group: &Glob<CollisionGroup>) -> bool {
        self.collisions
            .iter()
            .any(|c| c.other_group_index == group.index())
    }

    fn rigid_body<'a>(&self, app: &'a App) -> &'a RigidBody {
        self.pipeline.get(app).rigid_body(self.rigid_body_handle)
    }

    fn collider<'a>(&self, app: &'a App) -> &'a Collider {
        self.pipeline.get(app).collider(self.collider_handle)
    }

    fn collider_mut<'a>(&self, app: &'a mut App) -> &'a mut Collider {
        self.pipeline
            .get_mut(app)
            .rigid_body_and_collider_mut(self.rigid_body_handle, self.collider_handle)
            .1
    }

    fn default_rigid_body() -> RigidBody {
        RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .can_sleep(false)
            .translation(convert_vec2(Self::DEFAULT_POSITION))
            .build()
    }

    fn default_collider() -> Collider {
        let size = Self::DEFAULT_SIZE;
        ColliderBuilder::new(SharedShape::cuboid(size.x / 2., size.y / 2.))
            .enabled(false)
            .active_collision_types(ActiveCollisionTypes::all())
            .active_hooks(ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::MODIFY_SOLVER_CONTACTS)
            .mass(0.)
            .build()
    }
}

/// The shape of a [`Body2D`].
///
/// # Examples
///
/// See [`Body2D`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Shape2D {
    /// Rectangle shape.
    #[default]
    Rectangle,
    /// Circle shape.
    ///
    /// The diameter of the circle is the smallest size component of [`Body2D`].
    Circle,
}

fn convert_vector2(vector: Vector2<f32>) -> Vec2 {
    Vec2::new(vector.x, vector.y)
}

fn convert_vec2(vector: Vec2) -> Vector2<f32> {
    Vector2::new(vector.x, vector.y)
}
