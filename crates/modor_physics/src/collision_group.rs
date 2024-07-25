use crate::physics_hooks::PhysicsHooks;
use modor::{App, FromApp, Glob, RootNodeHandle};
use rapier2d::prelude::InteractionGroups;

/// A collision group that can interact with other collision groups.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// struct CollisionGroups {
///     wall: CollisionGroup,
///     ball: CollisionGroup,
///     paddle: CollisionGroup,
/// }
///
/// impl FromApp for CollisionGroups {
///     fn from_app(app: &mut App) -> Self {
///         let wall = CollisionGroup::new(app);
///         let ball = CollisionGroup::new(app);
///         ball.add_interaction(app, wall.glob(), CollisionType::Impulse(Impulse::new(1., 0.)));
///         let paddle = CollisionGroup::new(app);
///         paddle.add_interaction(app, wall.glob(), CollisionType::Impulse(Impulse::new(0., 0.)));
///         paddle.add_interaction(app, ball.glob(), CollisionType::Sensor);
///         Self {
///             wall,
///             ball,
///             paddle,
///         }   
///     }
/// }
///
/// impl RootNode for CollisionGroups {
///     fn update(&mut self, app: &mut App) {
///         self.wall.update(app);
///         self.ball.update(app);
///         self.paddle.update(app);
///     }
/// }
///
/// fn create_wall_body(app: &mut App, position: Vec2, size: Vec2) -> Body2D {
///     Body2D::new(app)
///         .with_position(position)
///         .with_size(size)
///         .with_collision_group(Some(app.get_mut::<CollisionGroups>().wall.glob().to_ref()))
/// }
/// ```
#[derive(Debug)]
pub struct CollisionGroup {
    pub(crate) glob: Glob<CollisionGroupGlob>,
    physics_hooks: RootNodeHandle<PhysicsHooks>,
}

impl CollisionGroup {
    /// Creates and register a new collision group.
    pub fn new(app: &mut App) -> Self {
        Self {
            glob: Glob::from_app(app),
            physics_hooks: app.handle::<PhysicsHooks>(),
        }
    }

    /// Updates the collision group.
    pub fn update(&mut self, app: &mut App) {
        let interactions = self
            .physics_hooks
            .get_mut(app)
            .interactions(self.glob.index());
        self.glob.get_mut(app).interactions = interactions;
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &Glob<CollisionGroupGlob> {
        &self.glob
    }

    /// Register an interaction of a given `type_` between the group and an `other` group.
    ///
    /// In case it already exists an interaction between these two groups, the collision type is
    /// overwritten.
    pub fn add_interaction(
        &self,
        app: &mut App,
        other: &Glob<CollisionGroupGlob>,
        type_: CollisionType,
    ) {
        self.physics_hooks
            .get_mut(app)
            .add_interaction(self.glob.index(), other.index(), type_);
    }
}

/// The global data of a [`CollisionGroup`].
#[derive(Debug)]
pub struct CollisionGroupGlob {
    pub(crate) interactions: InteractionGroups,
}

impl Default for CollisionGroupGlob {
    fn default() -> Self {
        Self {
            interactions: InteractionGroups::none(),
        }
    }
}

/// The collision behavior that should happen between two objects.
///
/// # Examples
///
/// See [`CollisionGroup`].
#[derive(Clone, Copy, PartialEq, Debug)]
#[non_exhaustive]
pub enum CollisionType {
    /// Collision should happen but it doesn't produce forces.
    Sensor,
    /// Collision should happen and it produces forces.
    ///
    /// Note that there is no effect if the body is not dynamic, or if its mass and angular inertia
    /// are equal to zero.
    Impulse(Impulse),
}

/// Properties of a collision of type [`CollisionType::Impulse`].
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
