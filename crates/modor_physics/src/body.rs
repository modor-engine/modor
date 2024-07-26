use crate::collisions::Collision2D;
use crate::pipeline::Pipeline;
use crate::user_data::ColliderUserData;
use crate::CollisionGroupGlob;
use modor::{App, Builder, FromApp, Glob, GlobRef, StateHandle};
use modor_math::Vec2;
use rapier2d::dynamics::{MassProperties, RigidBody, RigidBodyHandle, RigidBodyType};
use rapier2d::geometry::{
    ActiveCollisionTypes, Collider, ColliderBuilder, ColliderHandle, SharedShape,
};
use rapier2d::math::Rotation;
use rapier2d::na::{Point2, Vector2};
use rapier2d::pipeline::ActiveHooks;
use rapier2d::prelude::{InteractionGroups, RigidBodyBuilder};

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
/// struct Character {
///     body: Body2D,
/// }
///
/// impl Character {
///     fn new(app: &mut App, position: Vec2, group: &CollisionGroup) -> Self {
///         Self {
///             body: Body2D::new(app)
///                 .with_position(position)
///                 .with_size(Vec2::ONE * 0.2)
///                 .with_rotation(FRAC_PI_2)
///                 .with_collision_group(Some(group.glob().to_ref()))
///                 .with_shape(Shape2D::Circle)
///         }
///     }
///
///     fn update(&mut self, app: &mut App) {
///         self.body.velocity = app.get_mut::<CharacterDirection>().0 * 0.5;
///         self.body.update(app);
///         for collision in self.body.collisions() {
///             println!("Ball is colliding with body {}", collision.other_index);
///         }
///     }
/// }
/// ```
#[derive(Debug, Builder)]
pub struct Body2D {
    /// Position of the body in world units.
    ///
    /// Default is [`Vec2::ZERO`].
    #[builder(form(value))]
    pub position: Vec2,
    /// Size of the body in world units.
    ///
    /// Default is [`Vec2::ONE`].
    #[builder(form(value))]
    pub size: Vec2,
    /// Rotation of the body in radians.
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub rotation: f32,
    /// Linear velocity of the body in world units per second.
    ///
    /// Default is `Vec2::ZERO`.
    #[builder(form(value))]
    pub velocity: Vec2,
    /// Angular velocity of the body in radians per second.
    ///
    /// Has no effect if [`angular_inertia`](#structfield.angular_inertia) is `0.0`.
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub angular_velocity: f32,
    /// Force applied on the body.
    ///
    /// Has no effect if [`mass`](#structfield.mass) is `0.0`.
    ///
    /// The acceleration of the body corresponds to the force of the body divided by its mass.
    ///
    /// Default is [`Vec2::ZERO`].
    #[builder(form(value))]
    pub force: Vec2,
    /// Torque applied on the body.
    ///
    /// Has no effect if [`angular_inertia`](#structfield.angular_inertia) is `0.0`.
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub torque: f32,
    /// Mass of the body.
    ///
    /// A mass of zero is considered as infinite. In this case, force will not have any effect
    /// (even in case of collisions).
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub mass: f32,
    /// Angular inertia of the body.
    ///
    /// An angular inertia of zero is considered as infinite. In this case, torque will not have
    /// any effect (even in case of collisions).
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub angular_inertia: f32,
    /// Linear damping of the body.
    ///
    /// This coefficient is used to automatically slow down the translation of the body.
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub damping: f32,
    /// Angular damping of the body.
    ///
    /// This coefficient is used to automatically slow down the rotation of the body.
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub angular_damping: f32,
    /// Dominance of the body.
    ///
    /// In case of collision between two bodies, if both bodies have a different dominance
    /// group, then collision forces will only be applied on the body with the smallest dominance.
    ///
    /// Has no effect if [`collision_group`](#structfield.collision_group) is `None`.
    ///
    /// Default is `0`.
    #[builder(form(value))]
    pub dominance: i8,
    /// Whether Continuous Collision Detection is enabled for the body.
    ///
    /// This option is used to detect a collision even if the body moves too fast.
    /// CCD is performed using motion-clamping, which means each fast-moving body with CCD enabled
    /// will be stopped at the moment of their first contact. Both angular and translational motions
    /// are taken into account.
    ///
    /// Note that CCD require additional computation, so it is recommended to enable it only for
    /// bodies that are expected to move fast.
    ///
    /// Has no effect if [`collision_group`](#structfield.collision_group) is `None`.
    ///
    /// Default is `false`.
    #[builder(form(value))]
    pub is_ccd_enabled: bool,
    /// Collision group of the collider.
    ///
    /// Note that the collisions may not be updated when only the [`size`](#structfield.size) is
    /// changed. However, it is ensured the collision is detected when updating
    /// the [`position`](#structfield.position) or the [`rotation`](#structfield.rotation).
    ///
    /// Default is `None` (no collision detection is performed).
    #[builder(form(value))]
    pub collision_group: Option<GlobRef<CollisionGroupGlob>>,
    /// The shape of the body used to detect collisions.
    ///
    /// Default is [`Shape2D::Rectangle`].
    #[builder(form(value))]
    pub shape: Shape2D,
    collisions: Vec<Collision2D>,
    glob: Glob<Body2DGlob>,
    pipeline: StateHandle<Pipeline>,
}

impl Body2D {
    const DEFAULT_POSITION: Vec2 = Vec2::ZERO;
    const DEFAULT_SIZE: Vec2 = Vec2::ONE;

    /// Creates a new body.
    pub fn new(app: &mut App) -> Self {
        let active_hooks = ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::MODIFY_SOLVER_CONTACTS;
        let pipeline_handle = app.handle::<Pipeline>();
        let (rigid_body_handle, collider_handle) = pipeline_handle.get_mut(app).register_body(
            Self::default_rigid_body(Self::DEFAULT_POSITION),
            Self::default_collider(Self::DEFAULT_SIZE, active_hooks),
        );
        let glob = Glob::<Body2DGlob>::from_app(app);
        let glob_ref = glob.get_mut(app);
        glob_ref.rigid_body_handle = Some(rigid_body_handle);
        glob_ref.collider_handle = Some(collider_handle);
        Self {
            position: Self::DEFAULT_POSITION,
            size: Self::DEFAULT_SIZE,
            rotation: 0.,
            velocity: Vec2::ZERO,
            angular_velocity: 0.,
            force: Vec2::ZERO,
            torque: 0.,
            mass: 0.,
            angular_inertia: 0.,
            damping: 0.,
            angular_damping: 0.,
            dominance: 0,
            is_ccd_enabled: false,
            collision_group: None,
            shape: Shape2D::Rectangle,
            collisions: vec![],
            glob,
            pipeline: pipeline_handle,
        }
    }

    /// Updates the body.
    pub fn update(&mut self, app: &mut App) {
        let glob = self.glob.get(app);
        let changes = Body2DChanges::new(self, glob);
        let rigid_body_handle = glob
            .rigid_body_handle
            .expect("internal error: no body handle");
        let collider_handle = glob
            .collider_handle
            .expect("internal error: no collider handle");
        let interaction_groups = self
            .collision_group
            .as_ref()
            .map_or_else(InteractionGroups::none, |g| g.get(app).interactions);
        let pipeline = self.pipeline.get_mut(app);
        let rigid_body = pipeline.rigid_body_mut(rigid_body_handle);
        self.update_from_rigid_body(rigid_body, changes);
        self.update_rigid_body(rigid_body, changes);
        let collider = pipeline.collider_mut(collider_handle);
        self.update_collider(collider, changes, interaction_groups);
        self.collisions = pipeline.collisions(collider_handle).to_vec();
        let glob = self.glob.get_mut(app);
        self.update_glob(glob);
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &Glob<Body2DGlob> {
        &self.glob
    }

    /// Returns the detected collisions.
    pub fn collisions(&self) -> &[Collision2D] {
        &self.collisions
    }

    /// Returns the detected collisions with another body from the specific collision `group`.
    pub fn collisions_with(
        &self,
        group: &Glob<CollisionGroupGlob>,
    ) -> impl Iterator<Item = Collision2D> + '_ {
        let group_index = group.index();
        self.collisions
            .iter()
            .copied()
            .filter(move |collision| collision.other_group_index == group_index)
    }

    /// Returns whether the body collides with a body inside `group`.
    pub fn is_colliding_with(&self, group: &Glob<CollisionGroupGlob>) -> bool {
        self.collisions
            .iter()
            .any(|c| c.other_group_index == group.index())
    }

    fn default_collider(size: Vec2, active_hooks: ActiveHooks) -> Collider {
        ColliderBuilder::new(SharedShape::cuboid(size.x / 2., size.y / 2.))
            .enabled(false)
            .active_collision_types(ActiveCollisionTypes::all())
            .active_hooks(active_hooks)
            .mass(0.)
            .build()
    }

    fn default_rigid_body(position: Vec2) -> RigidBody {
        RigidBodyBuilder::new(RigidBodyType::Dynamic)
            .can_sleep(false)
            .translation(Vector2::new(position.x, position.y))
            .build()
    }

    fn update_glob(&self, glob: &mut Body2DGlob) {
        glob.position = self.position;
        glob.size = self.size;
        glob.rotation = self.rotation;
        glob.velocity = self.velocity;
        glob.angular_velocity = self.angular_velocity;
        glob.force = self.force;
        glob.torque = self.torque;
        glob.mass = self.mass;
        glob.angular_inertia = self.angular_inertia;
        glob.shape = self.shape;
    }

    fn update_from_rigid_body(&mut self, rigid_body: &RigidBody, changes: Body2DChanges) {
        if !changes.is_position_changed {
            self.position.x = rigid_body.translation().x;
            self.position.y = rigid_body.translation().y;
        }
        if !changes.is_rotation_changed {
            self.rotation = rigid_body.rotation().angle();
        }
        if !changes.is_velocity_changed {
            self.velocity.x = rigid_body.linvel().x;
            self.velocity.y = rigid_body.linvel().y;
        }
        if !changes.is_angular_velocity_changed {
            self.angular_velocity = rigid_body.angvel();
        }
        if !changes.is_force_changed {
            self.force.x = rigid_body.user_force().x;
            self.force.y = rigid_body.user_force().y;
        }
        if !changes.is_torque_changed {
            self.torque = rigid_body.user_torque();
        }
    }

    fn update_rigid_body(&mut self, rigid_body: &mut RigidBody, changes: Body2DChanges) {
        rigid_body.set_translation(Vector2::new(self.position.x, self.position.y), true);
        rigid_body.set_rotation(Rotation::new(self.rotation), true);
        rigid_body.set_linvel(Vector2::new(self.velocity.x, self.velocity.y), true);
        rigid_body.set_angvel(self.angular_velocity, true);
        rigid_body.reset_forces(true);
        rigid_body.add_force(Vector2::new(self.force.x, self.force.y), true);
        rigid_body.reset_torques(true);
        rigid_body.add_torque(self.torque, true);
        if changes.is_mass_changed || changes.is_angular_inertia_changed {
            let mass = MassProperties::new(Point2::new(0., 0.), self.mass, self.angular_inertia);
            rigid_body.set_additional_mass_properties(mass, true);
        }
        rigid_body.set_linear_damping(self.damping);
        rigid_body.set_angular_damping(self.angular_damping);
        rigid_body.set_dominance_group(self.dominance);
        rigid_body.enable_ccd(self.is_ccd_enabled);
    }

    fn update_collider(
        &mut self,
        collider: &mut Collider,
        changes: Body2DChanges,
        interaction_groups: InteractionGroups,
    ) {
        if changes.is_size_changed || changes.is_shape_changed {
            collider.set_shape(match self.shape {
                Shape2D::Rectangle => SharedShape::cuboid(self.size.x / 2., self.size.y / 2.),
                Shape2D::Circle => SharedShape::ball(self.size.x.min(self.size.y) / 2.),
            });
        }
        let group_index = self
            .collision_group
            .as_ref()
            .map_or(0, |group| group.index());
        collider.user_data = ColliderUserData::new(self.glob.index(), group_index).into();
        collider.set_enabled(self.collision_group.is_some());
        collider.set_collision_groups(interaction_groups);
        collider.set_mass(0.);
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

/// The global data of a [`Body2D`].
#[derive(Debug)]
pub struct Body2DGlob {
    /// Position of the body in world units.
    pub position: Vec2,
    /// Size of the body in world units.
    pub size: Vec2,
    /// Rotation of the body in radians.
    pub rotation: f32,
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,
    pub(crate) collider_handle: Option<ColliderHandle>,
    velocity: Vec2,
    angular_velocity: f32,
    force: Vec2,
    torque: f32,
    mass: f32,
    angular_inertia: f32,
    shape: Shape2D,
}

impl Default for Body2DGlob {
    fn default() -> Self {
        Self {
            position: Body2D::DEFAULT_POSITION,
            size: Body2D::DEFAULT_SIZE,
            rotation: 0.,
            rigid_body_handle: None,
            collider_handle: None,
            velocity: Vec2::ZERO,
            angular_velocity: 0.,
            force: Vec2::ZERO,
            torque: 0.,
            mass: 0.,
            angular_inertia: 0.,
            shape: Shape2D::Rectangle,
        }
    }
}

#[allow(clippy::struct_field_names, clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
struct Body2DChanges {
    is_position_changed: bool,
    is_size_changed: bool,
    is_rotation_changed: bool,
    is_velocity_changed: bool,
    is_angular_velocity_changed: bool,
    is_force_changed: bool,
    is_torque_changed: bool,
    is_mass_changed: bool,
    is_angular_inertia_changed: bool,
    is_shape_changed: bool,
}

impl Body2DChanges {
    #[allow(clippy::float_cmp)]
    fn new(body: &Body2D, old_body: &Body2DGlob) -> Self {
        Self {
            is_position_changed: body.position != old_body.position,
            is_size_changed: body.size != old_body.size,
            is_rotation_changed: body.rotation != old_body.rotation,
            is_velocity_changed: body.velocity != old_body.velocity,
            is_angular_velocity_changed: body.angular_velocity != old_body.angular_velocity,
            is_force_changed: body.force != old_body.force,
            is_torque_changed: body.torque != old_body.torque,
            is_mass_changed: body.mass != old_body.mass,
            is_angular_inertia_changed: body.angular_inertia != old_body.angular_inertia,
            is_shape_changed: body.shape != old_body.shape,
        }
    }
}
