use crate::pipeline::Pipeline;
use crate::user_data::ColliderUserData;
use crate::{Collision2D, CollisionGroup};
use modor::{App, FromApp, Glob, GlobRef, GlobUpdater, Global, StateHandle};
use modor_math::Vec2;
use rapier2d::dynamics::{
    MassProperties, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodyType,
};
use rapier2d::geometry::{
    ActiveCollisionTypes, Collider, ColliderBuilder, ColliderHandle, SharedShape,
};
use rapier2d::math::Rotation;
use rapier2d::na::{Point2, Vector2};
use rapier2d::pipeline::ActiveHooks;
use std::marker::PhantomData;

// TODO: split in multiple modules

macro_rules! doc {
    (position) => {
        "Position of the body in world units.<br>\
        Default is [`Vec2::ZERO`]."
    };
    (size) => {
        "Size of the body in world units.<br>\
        Default is [`Vec2::ONE`]."
    };
    (rotation) => {
        "Rotation of the body in radians.<br>\
        Default is `0.0`."
    };
    (velocity) => {
        "Linear velocity of the body in world units per second.<br>\
        Default is [`Vec2::ZERO`]."
    };
    (angular_velocity) => {
        "Angular velocity of the body in radians per second.<br>\
        Has no effect if the [`angular_inertia`](Body2D::angular_inertia) is `0.0`.<br>\
        Default is `0.0`."
    };
    (force) => {
        "Force applied on the body.<br>\
        Has no effect if the [`mass`](Body2D::mass) is `0.0`.<br>\
        The acceleration of the body corresponds to the force of the body divided by its mass.<br>\
        Default is [`Vec2::ZERO`]."
    };
    (torque) => {
        "Torque applied on the body.<br>\
        Has no effect if the [`angular_inertia`](Body2D::angular_inertia) is `0.0`.<br>\
        Default is `0.0`."
    };
    (mass) => {
        "Angular inertia of the body.<br>\
        An angular inertia of zero is considered as infinite. In this case, torque will not have
        any effect (even in case of collisions).<br>\
        Default is `0.0`."
    };
    (angular_inertia) => {
        "Angular inertia of the body.<br>\
        An angular inertia of zero is considered as infinite. In this case, torque will not have
        any effect (even in case of collisions).<br>\
        Default is `0.0`."
    };
    (damping) => {
        "Linear damping of the body.<br>\
        This coefficient is used to automatically slow down the translation of the body.<br>\
        Default is `0.0`."
    };
    (angular_damping) => {
        "Angular damping of the body.<br>\
        This coefficient is used to automatically slow down the rotation of the body.<br>\
        Default is `0.0`."
    };
    (dominance) => {
        "Dominance of the body.<br>\
        In case of collision between two bodies, if both bodies have a different dominance
        group, then collision forces will only be applied on the body with the smallest dominance.<br>\
        Has no effect if the [`collision_group`](Body2D::collision_group) is `None`.<br>\
        Default is `0`."
    };
    (is_ccd_enabled) => {
        "Whether Continuous Collision Detection is enabled for the body.<br>\
        This option is used to detect a collision even if the body moves too fast.
        CCD is performed using motion-clamping, which means each fast-moving body with CCD enabled
        will be stopped at the moment of their first contact. Both angular and translational motions
        are taken into account.<br>\
        Note that CCD require additional computation, so it is recommended to enable it only for
        bodies that are expected to move fast.<br>\
        Has no effect if [`collision_group`](#structfield.collision_group) is `None`.<br>\
        Default is `false`."
    };
    (collision_group) => {
        "Collision group of the collider.<br>\
        Note that the collisions may not be updated when only the [`size`](Body2D::size) is
        changed. However, it is ensured the collision is detected when updating
        the [`position`](Body2D::position) or the [`rotation`](Body2D::rotation).<br>\
        Default is `None` (no collision detection is performed)."
    };
    (shape) => {
        "The shape of the body used to detect collisions.<br>\
        Default is [`Shape2D::Rectangle`]."
    };
}

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
#[derive(Debug, GlobUpdater)]
pub struct Body2D {
    pub(crate) rigid_body_handle: RigidBodyHandle,
    pub(crate) collider_handle: ColliderHandle,
    #[doc = doc!(collision_group)]
    #[updater(field, for_field = "default")]
    pub(crate) collision_group: Option<GlobRef<CollisionGroup>>,
    pub(crate) collisions: Vec<Collision2D>,
    pipeline: StateHandle<Pipeline>,
    #[doc = doc!(position)]
    #[updater(inner_type, field, for_field = "Body2D::position")]
    position: PhantomData<Vec2>,
    #[doc = doc!(size)]
    #[updater(field, for_field = "default")]
    size: Vec2,
    #[doc = doc!(rotation)]
    #[updater(inner_type, field, for_field = "Body2D::rotation")]
    rotation: PhantomData<f32>,
    #[doc = doc!(velocity)]
    #[updater(inner_type, field, for_field = "Body2D::velocity")]
    velocity: PhantomData<Vec2>,
    #[doc = doc!(angular_velocity)]
    #[updater(inner_type, field, for_field = "Body2D::angular_velocity")]
    angular_velocity: PhantomData<f32>,
    #[doc = doc!(force)]
    #[updater(inner_type, field, for_field = "Body2D::force")]
    force: PhantomData<Vec2>,
    #[doc = doc!(torque)]
    #[updater(inner_type, field, for_field = "Body2D::torque")]
    torque: PhantomData<f32>,
    #[doc = doc!(mass)]
    #[updater(field, for_field = "default")]
    mass: f32, // stored locally so that Body2D::mass() gives immediately the new value
    #[doc = doc!(angular_inertia)]
    #[updater(field, for_field = "default")]
    angular_inertia: f32, // stored locally so that Body2D::angular_inertia() gives immediately the new value
    #[doc = doc!(damping)]
    #[updater(inner_type, field, for_field = "Body2D::damping")]
    damping: PhantomData<f32>,
    #[doc = doc!(angular_damping)]
    #[updater(inner_type, field, for_field = "Body2D::angular_damping")]
    angular_damping: PhantomData<f32>,
    #[doc = doc!(dominance)]
    #[updater(inner_type, field, for_field = "Body2D::dominance")]
    dominance: PhantomData<i8>,
    #[doc = doc!(is_ccd_enabled)]
    #[updater(inner_type, field, for_field = "Body2D::is_ccd_enabled")]
    is_ccd_enabled: PhantomData<bool>,
    #[doc = doc!(shape)]
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

    #[doc=doc!(position)]
    pub fn position(&self, app: &App) -> Vec2 {
        convert_vector2(*self.rigid_body(app).translation())
    }

    #[doc=doc!(size)]
    pub fn size(&self) -> Vec2 {
        self.size
    }

    #[doc=doc!(rotation)]
    pub fn rotation(&self, app: &App) -> f32 {
        self.rigid_body(app).rotation().angle()
    }

    #[doc=doc!(velocity)]
    pub fn velocity(&self, app: &App) -> Vec2 {
        convert_vector2(*self.rigid_body(app).linvel())
    }

    #[doc=doc!(angular_velocity)]
    pub fn angular_velocity(&self, app: &App) -> f32 {
        self.rigid_body(app).angvel()
    }

    #[doc=doc!(force)]
    pub fn force(&self, app: &App) -> Vec2 {
        convert_vector2(self.rigid_body(app).user_force())
    }

    #[doc=doc!(torque)]
    pub fn torque(&self, app: &App) -> f32 {
        self.rigid_body(app).user_torque()
    }

    // TODO: add Getters trait?
    #[doc=doc!(mass)]
    pub fn mass(&self) -> f32 {
        self.mass
    }

    // TODO: add Getters trait?
    #[doc=doc!(angular_inertia)]
    pub fn angular_inertia(&self) -> f32 {
        self.angular_inertia
    }

    #[doc=doc!(damping)]
    pub fn damping(&self, app: &App) -> f32 {
        self.rigid_body(app).linear_damping()
    }

    #[doc=doc!(angular_damping)]
    pub fn angular_damping(&self, app: &App) -> f32 {
        self.rigid_body(app).angular_damping()
    }

    #[doc=doc!(dominance)]
    pub fn dominance(&self, app: &App) -> i8 {
        self.rigid_body(app).dominance_group()
    }

    #[doc=doc!(is_ccd_enabled)]
    pub fn is_ccd_enabled(&self, app: &App) -> bool {
        self.rigid_body(app).is_ccd_enabled()
    }

    // TODO: add Getters trait?
    #[doc=doc!(collision_group)]
    pub fn collision_group(&self) -> &Option<GlobRef<CollisionGroup>> {
        &self.collision_group
    }

    #[doc=doc!(shape)]
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

impl Body2DUpdater<'_> {
    /// Run the update.
    pub fn apply(mut self, app: &mut App) {
        let (rigid_body, collider, size, mass, angular_inertia) = self.update_glob(app);
        self.update_collision_group(collider);
        self.update_position(rigid_body);
        self.update_size(collider);
        self.update_rotation(rigid_body);
        self.update_velocity(rigid_body);
        self.update_angular_velocity(rigid_body);
        self.update_force(rigid_body);
        self.update_torque(rigid_body);
        self.update_mass_and_angular_inertia(rigid_body, mass, angular_inertia);
        self.update_damping(rigid_body);
        self.update_angular_damping(rigid_body);
        self.update_dominance(rigid_body);
        self.update_ccd_enabled(rigid_body);
        self.update_shape(collider, size);
    }

    // TODO: avoid returning too many items (e.g. by taking the glob)
    fn update_glob<'a>(
        &mut self,
        app: &'a mut App,
    ) -> (&'a mut RigidBody, &'a mut Collider, Vec2, f32, f32) {
        let glob = self.glob.get_mut(app);
        let collision_group = self.collision_group.clone();
        modor::update_field(&mut glob.collision_group, collision_group, &mut false);
        modor::update_field(&mut glob.size, self.size, &mut false);
        modor::update_field(&mut glob.mass, self.mass, &mut false);
        modor::update_field(&mut glob.angular_inertia, self.angular_inertia, &mut false);
        let rigid_body_handle = glob.rigid_body_handle;
        let collider_handle = glob.collider_handle;
        let pipeline = glob.pipeline;
        let size = glob.size;
        let mass = glob.mass;
        let angular_inertia = glob.angular_inertia;
        let (rigid_body, collider) = pipeline
            .get_mut(app)
            .rigid_body_and_collider_mut(rigid_body_handle, collider_handle);
        (rigid_body, collider, size, mass, angular_inertia)
    }

    fn update_collision_group(&mut self, collider: &mut Collider) {
        if let Some(collision_group) = self.collision_group.take() {
            let group_index = collision_group
                .as_ref()
                .map_or(usize::MAX, |group| group.index());
            collider.user_data = ColliderUserData::new(self.glob.index(), group_index).into();
            collider.set_enabled(collision_group.is_some());
        }
    }

    fn update_position(&mut self, rigid_body: &mut RigidBody) {
        if let Some(position) = self.position {
            rigid_body.set_translation(convert_vec2(position), true);
        }
    }

    fn update_size(&mut self, collider: &mut Collider) {
        if let Some(size) = self.size {
            let shape = collider.shape_mut();
            if let Some(shape) = shape.as_cuboid_mut() {
                shape.half_extents = convert_vec2(size / 2.);
            } else if let Some(shape) = shape.as_ball_mut() {
                shape.radius = size.x.min(size.y) / 2.;
            } else {
                unreachable!("internal error: unsupported body shape")
            }
            collider.set_mass(0.);
        }
    }

    fn update_rotation(&mut self, rigid_body: &mut RigidBody) {
        if let Some(rotation) = self.rotation {
            rigid_body.set_rotation(Rotation::new(rotation), true);
        }
    }

    fn update_velocity(&mut self, rigid_body: &mut RigidBody) {
        if let Some(velocity) = self.velocity {
            rigid_body.set_linvel(convert_vec2(velocity), true);
        }
    }

    fn update_angular_velocity(&mut self, rigid_body: &mut RigidBody) {
        if let Some(angular_velocity) = self.angular_velocity {
            rigid_body.set_angvel(angular_velocity, true);
        }
    }

    fn update_force(&mut self, rigid_body: &mut RigidBody) {
        if let Some(force) = self.force {
            rigid_body.reset_forces(true);
            rigid_body.add_force(convert_vec2(force), true);
        }
    }

    fn update_torque(&mut self, rigid_body: &mut RigidBody) {
        if let Some(torque) = self.torque {
            rigid_body.reset_torques(true);
            rigid_body.add_torque(torque, true);
        }
    }

    fn update_mass_and_angular_inertia(
        &mut self,
        rigid_body: &mut RigidBody,
        mass: f32,
        angular_inertia: f32,
    ) {
        if self.mass.is_some() || self.angular_inertia.is_some() {
            let properties = MassProperties::new(Point2::new(0., 0.), mass, angular_inertia);
            rigid_body.set_additional_mass_properties(properties, true);
        }
    }

    fn update_damping(&mut self, rigid_body: &mut RigidBody) {
        if let Some(damping) = self.damping {
            rigid_body.set_linear_damping(damping);
        }
    }

    fn update_angular_damping(&mut self, rigid_body: &mut RigidBody) {
        if let Some(angular_damping) = self.angular_damping {
            rigid_body.set_angular_damping(angular_damping);
        }
    }

    fn update_dominance(&mut self, rigid_body: &mut RigidBody) {
        if let Some(dominance) = self.dominance {
            rigid_body.set_dominance_group(dominance);
        }
    }

    fn update_ccd_enabled(&mut self, rigid_body: &mut RigidBody) {
        if let Some(is_ccd_enabled) = self.is_ccd_enabled {
            rigid_body.enable_ccd(is_ccd_enabled);
        }
    }

    fn update_shape(&mut self, collider: &mut Collider, size: Vec2) {
        if let Some(shape) = self.shape {
            collider.set_shape(match shape {
                Shape2D::Rectangle => SharedShape::cuboid(size.x / 2., size.y / 2.),
                Shape2D::Circle => SharedShape::ball(size.x.min(size.y) / 2.),
            });
            collider.set_mass(0.);
        }
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
