use crate::body_register::Body2DRegister;
use crate::CollisionGroup;
use approx::AbsDiffEq;
use modor::{Context, NoVisit, Node};
use modor_internal::index::Index;
use modor_math::Vec2;
use rapier2d::dynamics::{MassProperties, RigidBody, RigidBodyType};
use rapier2d::geometry::{ActiveCollisionTypes, Collider, ColliderBuilder, SharedShape};
use rapier2d::math::Rotation;
use rapier2d::na::{Point2, Vector2};
use rapier2d::pipeline::ActiveHooks;
use rapier2d::prelude::{ContactManifold, InteractionGroups, RigidBodyBuilder};
use std::sync::Arc;

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
/// #[derive(Default, RootNode, Node, NoVisit)]
/// struct CharacterDirection(Vec2);
///
/// #[derive(Visit)]
/// struct Character {
///     body: Body2D,
/// }
///
/// impl Node for Character {
///     fn on_enter(&mut self, ctx: &mut Context<'_>) {
///         self.body.velocity = ctx.root::<CharacterDirection>().0 * 0.5;
///     }
///
///     fn on_exit(&mut self, ctx: &mut Context<'_>) {
///         for collision in self.body.collisions() {
///             println!("Ball is colliding with body {}", collision.other_index);
///         }
///     }
/// }
///
/// impl Character {
///     fn new(ctx: &mut Context<'_>, position: Vec2, group: &CollisionGroup) -> Self {
///         let mut body = Body2D::new(ctx, position, Vec2::ONE * 0.2);
///         body.rotation = FRAC_PI_2;
///         body.collision_group = Some(group.clone());
///         body.shape = Shape2D::Circle;
///         Self { body }
///     }
/// }
/// ```
#[derive(Debug, NoVisit)]
pub struct Body2D {
    /// Position of the body in world units.
    pub position: Vec2,
    /// Size of the body in world units.
    pub size: Vec2,
    /// Rotation of the body in radians.
    ///
    /// Default value is `0.0`.
    pub rotation: f32,
    /// Linear velocity of the body in world units per second.
    ///
    /// Default value is `Vec2::ZERO`.
    pub velocity: Vec2,
    /// Angular velocity of the body in radians per second.
    ///
    /// Has no effect if [`angular_inertia`](#structfield.angular_inertia) is `0.0`.
    ///
    /// Default value is `0.0`.
    pub angular_velocity: f32,
    /// Force applied on the body.
    ///
    /// Has no effect if [`mass`](#structfield.mass) is `0.0`.
    ///
    /// The acceleration of the body corresponds to the force of the body divided by its mass.
    ///
    /// Default value is [`Vec2::ZERO`].
    pub force: Vec2,
    /// Torque applied on the body.
    ///
    /// Has no effect if [`angular_inertia`](#structfield.angular_inertia) is `0.0`.
    ///
    /// Default value is `0.0`.
    pub torque: f32,
    /// Mass of the body.
    ///
    /// A mass of zero is considered as infinite. In this case, force will not have any effect
    /// (even in case of collisions).
    ///
    /// Default value is `0.0`.
    pub mass: f32,
    /// Angular inertia of the body.
    ///
    /// An angular inertia of zero is considered as infinite. In this case, torque will not have
    /// any effect (even in case of collisions).
    ///
    /// Default value is `0.0`.
    pub angular_inertia: f32,
    /// Linear damping of the body.
    ///
    /// This coefficient is used to automatically slow down the translation of the body.
    ///
    /// Default value is `0.0`.
    pub damping: f32,
    /// Angular damping of the body.
    ///
    /// This coefficient is used to automatically slow down the rotation of the body.
    ///
    /// Default value is `0.0`.
    pub angular_damping: f32,
    /// Dominance of the body.
    ///
    /// In case of collision between two bodies, if both bodies have a different dominance
    /// group, then collision forces will only be applied on the body with the smallest dominance.
    ///
    /// Has no effect if [`collision_group`](#structfield.collision_group) is `None`.
    ///
    /// Default value is `0`.
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
    /// Has no effect if [`collision_group`](#structfield.collision_group) is [`None`].
    ///
    /// Default value is `false`.
    pub is_ccd_enabled: bool,
    /// Collision group of the collider.
    ///
    /// Note that the collisions may not be updated when only the [`size`](#structfield.size) is
    /// changed. However, it is ensured the collision is detected when updating
    /// the [`position`](#structfield.position) or the [`rotation`](#structfield.rotation).
    ///
    /// Default value is `None` (no collision detection is performed).
    pub collision_group: Option<CollisionGroup>,
    /// The shape of the body used to detect collisions.
    ///
    /// Default value is [`Shape2D::Rectangle`].
    pub shape: Shape2D,
    collisions: Vec<Collision2D>,
    index: Arc<Index>,
    old_position: Vec2,
    old_size: Vec2,
    old_rotation: f32,
    old_velocity: Vec2,
    old_angular_velocity: f32,
    old_force: Vec2,
    old_torque: f32,
    old_mass: f32,
    old_angular_inertia: f32,
    old_shape: Shape2D,
}

impl Node for Body2D {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let interaction_groups = self
            .collision_group
            .as_ref()
            .map_or_else(InteractionGroups::none, |g| g.interaction_groups(ctx));
        let pipeline = ctx.root::<Body2DRegister>();
        let rigid_body = pipeline.rigid_body_mut(&self.index);
        self.update_from_rigid_body(rigid_body);
        self.update_rigid_body(rigid_body);
        let collider = pipeline.collider_mut(&self.index);
        self.update_collider(collider, interaction_groups);
        pipeline.set_size(&self.index, self.size);
        self.collisions = pipeline.collisions(&self.index);
        self.reset_old();
    }
}

impl Body2D {
    /// Creates and register a new body.
    pub fn new(ctx: &mut Context<'_>, position: Vec2, size: Vec2) -> Self {
        let active_hooks = ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::MODIFY_SOLVER_CONTACTS;
        let resource = ctx.root::<Body2DRegister>().register_body(
            RigidBodyBuilder::new(RigidBodyType::Dynamic)
                .can_sleep(false)
                .translation(Vector2::new(position.x, position.y))
                .build(),
            ColliderBuilder::new(SharedShape::cuboid(size.x / 2., size.y / 2.))
                .enabled(false)
                .active_collision_types(ActiveCollisionTypes::all())
                .active_hooks(active_hooks)
                .mass(0.)
                .build(),
            size,
        );
        Self {
            position,
            size,
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
            index: Arc::new(resource),
            old_position: position,
            old_size: size,
            old_rotation: 0.,
            old_velocity: Vec2::ZERO,
            old_angular_velocity: 0.,
            old_force: Vec2::ZERO,
            old_torque: 0.,
            old_mass: 0.,
            old_angular_inertia: 0.,
            old_shape: Shape2D::Rectangle,
        }
    }

    /// Returns the unique body index.
    pub fn index(&self) -> Body2DIndex {
        Body2DIndex(self.index.clone())
    }

    /// Returns the detected collisions.
    pub fn collisions(&self) -> &[Collision2D] {
        &self.collisions
    }

    /// Returns whether the body collides with a body inside `group`.
    pub fn is_colliding_with(&self, group: &CollisionGroup) -> bool {
        self.collisions
            .iter()
            .any(|c| c.other_group_index == group.index())
    }

    #[allow(clippy::float_cmp)]
    fn update_from_rigid_body(&mut self, rigid_body: &RigidBody) {
        if self.position == self.old_position {
            self.position.x = rigid_body.translation().x;
            self.position.y = rigid_body.translation().y;
        }
        if self.rotation == self.old_rotation {
            self.rotation = rigid_body.rotation().angle();
        }
        if self.velocity == self.old_velocity {
            self.velocity.x = rigid_body.linvel().x;
            self.velocity.y = rigid_body.linvel().y;
        }
        if self.angular_velocity == self.old_angular_velocity {
            self.angular_velocity = rigid_body.angvel();
        }
        if self.force == self.old_force {
            self.force.x = rigid_body.user_force().x;
            self.force.y = rigid_body.user_force().y;
        }
        if self.torque == self.old_torque {
            self.torque = rigid_body.user_torque();
        }
    }

    #[allow(clippy::float_cmp)]
    fn update_rigid_body(&mut self, rigid_body: &mut RigidBody) {
        rigid_body.set_translation(Vector2::new(self.position.x, self.position.y), true);
        rigid_body.set_rotation(Rotation::new(self.rotation), true);
        rigid_body.set_linvel(Vector2::new(self.velocity.x, self.velocity.y), true);
        rigid_body.set_angvel(self.angular_velocity, true);
        rigid_body.reset_forces(true);
        rigid_body.add_force(Vector2::new(self.force.x, self.force.y), true);
        rigid_body.reset_torques(true);
        rigid_body.add_torque(self.torque, true);
        if self.mass != self.old_mass || self.angular_inertia != self.old_angular_inertia {
            let mass = MassProperties::new(Point2::new(0., 0.), self.mass, self.angular_inertia);
            rigid_body.set_additional_mass_properties(mass, true);
        }
        rigid_body.set_linear_damping(self.damping);
        rigid_body.set_angular_damping(self.angular_damping);
        rigid_body.set_dominance_group(self.dominance);
        rigid_body.enable_ccd(self.is_ccd_enabled);
    }

    fn update_collider(&mut self, collider: &mut Collider, interaction_groups: InteractionGroups) {
        if self.size != self.old_size || self.shape != self.old_shape {
            collider.set_shape(match self.shape {
                Shape2D::Rectangle => SharedShape::cuboid(self.size.x / 2., self.size.y / 2.),
                Shape2D::Circle => SharedShape::ball(self.size.x.min(self.size.y) / 2.),
            });
        }
        collider.user_data = self
            .collision_group
            .as_ref()
            .map_or_else(|| 0, CollisionGroup::index) as u128;
        collider.set_enabled(self.collision_group.is_some());
        collider.set_collision_groups(interaction_groups);
        collider.set_mass(0.);
    }

    fn reset_old(&mut self) {
        self.old_position = self.position;
        self.old_size = self.size;
        self.old_rotation = self.rotation;
        self.old_velocity = self.velocity;
        self.old_angular_velocity = self.angular_velocity;
        self.old_force = self.force;
        self.old_torque = self.torque;
        self.old_mass = self.mass;
        self.old_angular_inertia = self.angular_inertia;
        self.old_shape = self.shape;
    }
}

/// The unique index of a [`Body2D`].
#[derive(Debug)]
pub struct Body2DIndex(Arc<Index>);

impl Body2DIndex {
    /// Returns the index as a [`usize`].
    ///
    /// Note that in case the [`Body2D`] and all associated [`Body2DIndex`]s are dropped, this index
    /// can be reused for a new body.
    pub fn as_usize(&self) -> usize {
        self.0.value()
    }
}

/// The shape of a [`Body2D`].
///
/// # Examples
///
/// See [`Body2D`].
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape2D {
    /// Rectangle shape.
    #[default]
    Rectangle,
    /// Circle shape.
    ///
    /// The diameter of the circle is the smallest size component of [`Body2D`].
    Circle,
}

/// A detected collision.
///
/// # Examples
///
/// See [`Body2D`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Collision2D {
    /// Index of the collided body.
    pub other_index: usize,
    /// Index of the collision group corresponding to the collided body.
    pub other_group_index: usize,
    /// Penetration of the body into the collided one in world units.
    ///
    /// Penetration vector starts at other body edge and ends at current body deepest point.
    pub penetration: Vec2,
    /// Position of the collision in world units.
    ///
    /// This position corresponds to the deepest point of the current body inside the other body.
    /// If more than two points have the same depth, then the collision position is the average
    /// of these points.
    pub position: Vec2,
}

impl Collision2D {
    pub(crate) fn new(
        is_collider2: bool,
        other_index: usize,
        other_group_index: usize,
        collider: &Collider,
        manifold: &ContactManifold,
    ) -> Self {
        let max_distance = manifold.points.iter().map(|p| -p.dist).fold(0., f32::max);
        Self {
            other_index,
            other_group_index,
            penetration: Self::penetration(is_collider2, manifold, max_distance),
            position: Self::position(is_collider2, collider, manifold, max_distance),
        }
    }

    fn penetration(is_collider2: bool, manifold: &ContactManifold, max_distance: f32) -> Vec2 {
        Vec2::new(manifold.data.normal.x, manifold.data.normal.y)
            * max_distance
            * if is_collider2 { -1. } else { 1. }
    }

    #[allow(clippy::cast_precision_loss)]
    fn position(
        is_collider2: bool,
        collider: &Collider,
        manifold: &ContactManifold,
        max_distance: f32,
    ) -> Vec2 {
        manifold
            .points
            .iter()
            .filter(|d| d.dist.abs_diff_eq(&-max_distance, f32::EPSILON))
            .map(|p| if is_collider2 { p.local_p2 } else { p.local_p1 })
            .map(|p| Self::local_to_global_position(p, collider))
            .sum::<Vec2>()
            / manifold
                .points
                .iter()
                .filter(|d| d.dist.abs_diff_eq(&-max_distance, 100. * f32::EPSILON))
                .count() as f32
    }

    fn local_to_global_position(local_positions: Point2<f32>, collider: &Collider) -> Vec2 {
        Vec2::new(local_positions.x, local_positions.y).with_rotation(collider.rotation().angle())
            + Vec2::new(collider.translation().x, collider.translation().y)
    }
}
