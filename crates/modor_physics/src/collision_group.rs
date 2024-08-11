use crate::physics_hooks::{CollisionType, PhysicsHooks};
use modor::{App, FromApp, Glob, Global};

/// A collision group that can interact with other collision groups.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// #[derive(FromApp)]
/// struct CollisionGroups {
///     wall: Glob<CollisionGroup>,
///     ball: Glob<CollisionGroup>,
///     paddle: Glob<CollisionGroup>,
/// }
///
/// impl State for CollisionGroups {
///     fn init(&mut self, app: &mut App) {
///         CollisionGroupUpdater::new(&self.ball)
///             .add_impulse(app, &self.wall, Impulse::new(1., 0.));
///         CollisionGroupUpdater::new(&self.paddle)
///             .add_impulse(app, &self.wall, Impulse::new(0., 0.))
///             .add_sensor(app, &self.ball);
///     }
/// }
///
/// fn init_wall(app: &mut App, body: &Glob<Body2D>) {
///     Body2DUpdater::default()
///         .collision_group(app.get_mut::<CollisionGroups>().wall.to_ref())
///         .apply(app, &body);
/// }
/// ```
#[derive(Debug, FromApp)]
#[non_exhaustive]
pub struct CollisionGroup {
    index: usize,
}

impl Global for CollisionGroup {
    fn init(&mut self, app: &mut App, index: usize) {
        self.index = index;
        app.get_mut::<PhysicsHooks>().register_group(index);
    }
}

/// An updater for [`CollisionGroup`].
pub struct CollisionGroupUpdater<'a> {
    glob: &'a Glob<CollisionGroup>,
}

impl<'a> CollisionGroupUpdater<'a> {
    /// Creates a new updater.
    pub fn new(glob: &'a Glob<CollisionGroup>) -> Self {
        Self { glob }
    }

    /// Register a sensor interaction between the group and an `other` group.
    ///
    /// The collisions will be detected but don't produce forces.
    ///
    /// In case it already exists an interaction between these two groups, the interaction is
    /// overwritten.
    pub fn add_sensor(&self, app: &mut App, other: &Glob<CollisionGroup>) -> &Self {
        app.get_mut::<PhysicsHooks>().add_interaction(
            self.glob.index(),
            other.index(),
            CollisionType::Sensor,
        );
        self
    }

    /// Register a sensor interaction between the group and an `other` group.
    ///
    /// The collisions will be detected and produce forces. Note that there is no effect if
    /// the body [`mass`](crate::Body2D::mass) and
    /// [`angular_inertia`](crate::Body2D::angular_inertia) are equal to zero.
    ///
    /// In case it already exists an interaction between these two groups, the interaction is
    /// overwritten.
    pub fn add_impulse(
        &self,
        app: &mut App,
        other: &Glob<CollisionGroup>,
        impulse: Impulse,
    ) -> &Self {
        app.get_mut::<PhysicsHooks>().add_interaction(
            self.glob.index(),
            other.index(),
            CollisionType::Impulse(impulse),
        );
        self
    }
}

/// Properties of an impulse interaction between two [`CollisionGroup`]s.
///
/// # Examples
///
/// See [`CollisionGroup`].
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Impulse {
    /// Restitution coefficient of the collision.
    ///
    /// A coefficient of `0.0` means that the bodies do not bounce off each other at all.<br>
    /// A coefficient of `1.0` means that the exit velocity magnitude is the same as the initial
    /// velocity along the contact normal.
    ///
    /// Default is `0.0`.
    pub restitution: f32,
    /// Friction coefficient of the collision.
    ///
    /// A coefficient of `0.0` means there is no friction (i.e. objects slide completely over each
    /// other).
    ///
    /// Default is `0.5`.
    pub friction: f32,
}

impl Default for Impulse {
    fn default() -> Self {
        Self {
            restitution: 0.,
            friction: 0.5,
        }
    }
}

impl Impulse {
    /// Creates a new impulse configuration.
    pub fn new(restitution: f32, friction: f32) -> Self {
        Self {
            restitution,
            friction,
        }
    }
}
