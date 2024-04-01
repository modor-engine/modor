use fxhash::FxHashMap;
use modor::{Context, NoVisit, Node, RootNode};
use modor_internal::index::{Index, IndexPool};
use rapier2d::geometry::SolverFlags;
use rapier2d::pipeline::{ContactModificationContext, PairFilterContext};
use rapier2d::prelude::{Group, InteractionGroups};
use std::sync::Arc;

#[derive(Debug, Default, RootNode, NoVisit)]
pub(crate) struct CollisionGroupRegister {
    ids: Arc<IndexPool>,
    collision_types: FxHashMap<(usize, usize), CollisionType>,
    interaction_groups: Vec<InteractionGroups>,
}

impl Node for CollisionGroupRegister {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        for index in self.ids.take_deleted_indexes() {
            self.interaction_groups[index] = InteractionGroups::none();
            self.collision_types
                .retain(|&(index1, index2), _| index == index1 || index == index2);
        }
        for group in &mut self.interaction_groups {
            group.filter = Group::empty();
        }
        for &(index1, index2) in self.collision_types.keys() {
            Self::add_filter(&mut self.interaction_groups, index1, index2);
        }
    }
}

impl rapier2d::pipeline::PhysicsHooks for CollisionGroupRegister {
    fn filter_contact_pair(&self, context: &PairFilterContext<'_>) -> Option<SolverFlags> {
        let group1_index = context.colliders[context.collider1].user_data as usize;
        let group2_index = context.colliders[context.collider2].user_data as usize;
        match self.collision_types.get(&(group1_index, group2_index))? {
            CollisionType::Sensor => Some(SolverFlags::empty()),
            CollisionType::Impulse(_) => Some(SolverFlags::COMPUTE_IMPULSES),
        }
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext<'_>) {
        let group1_index = context.colliders[context.collider1].user_data as usize;
        let group2_index = context.colliders[context.collider2].user_data as usize;
        if let Some(CollisionType::Impulse(impulse)) =
            self.collision_types.get(&(group1_index, group2_index))
        {
            for contact in context.solver_contacts.iter_mut() {
                contact.restitution = impulse.restitution;
                contact.friction = impulse.friction;
            }
        }
    }
}

impl CollisionGroupRegister {
    fn register(&mut self) -> Index {
        let index = self.ids.generate();
        for index in self.interaction_groups.len()..=index.value() {
            self.interaction_groups.push(InteractionGroups::new(
                Group::from(1 << (index % 32)),
                Group::empty(),
            ));
        }
        index
    }

    fn add_interaction(&mut self, index1: &Index, index2: &Index, type_: CollisionType) {
        self.collision_types
            .insert((index1.value(), index2.value()), type_);
        self.collision_types
            .insert((index2.value(), index1.value()), type_);
        Self::add_filter(&mut self.interaction_groups, index1.value(), index2.value());
    }

    fn add_filter(groups: &mut [InteractionGroups], index1: usize, index2: usize) {
        groups[index1].filter |= Group::from(1 << (index2 % 32));
        groups[index2].filter |= Group::from(1 << (index1 % 32));
    }
}

/// The reference to a collision group that can interact with other collision groups.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// #[derive(Node, NoVisit)]
/// struct CollisionGroups {
///     wall: CollisionGroup,
///     ball: CollisionGroup,
///     paddle: CollisionGroup,
/// }
///
/// impl RootNode for CollisionGroups {
///     fn on_create(ctx: &mut Context<'_>) -> Self {
///         let wall = CollisionGroup::new(ctx);
///         let ball = CollisionGroup::new(ctx);
///         ball.add_interaction(ctx, &wall, CollisionType::Impulse(Impulse::new(1., 0.)));
///         let paddle = CollisionGroup::new(ctx);
///         paddle.add_interaction(ctx, &wall, CollisionType::Impulse(Impulse::new(0., 0.)));
///         paddle.add_interaction(ctx, &ball, CollisionType::Sensor);
///         Self {
///             wall,
///             ball,
///             paddle,
///         }   
///     }
/// }
///
/// fn create_wall_body(ctx: &mut Context<'_>, position: Vec2, size: Vec2) -> Body2D {
///     let mut body = Body2D::new(ctx, position, size);
///     let groups = ctx.root::<CollisionGroups>();
///     body.collision_group = Some(groups.wall.clone());
///     body
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CollisionGroup {
    index: Arc<Index>,
}

impl CollisionGroup {
    /// Creates and register a new collision group.
    pub fn new(ctx: &mut Context<'_>) -> Self {
        Self {
            index: Arc::new(ctx.root::<CollisionGroupRegister>().register()),
        }
    }

    /// Returns the unique index of the collision group.
    ///
    /// Note that index of a dropped group can be reused for a new group.
    pub fn index(&self) -> usize {
        self.index.value()
    }

    /// Register an interaction of a given `type_` between the group and an `other` group.
    ///
    /// In case it already exists an interaction between these two groups, the collision type is
    /// overwritten.
    pub fn add_interaction(&self, ctx: &mut Context<'_>, other: &Self, type_: CollisionType) {
        ctx.root::<CollisionGroupRegister>()
            .add_interaction(&self.index, &other.index, type_);
    }

    pub(crate) fn interaction_groups(&self, ctx: &mut Context<'_>) -> InteractionGroups {
        ctx.root::<CollisionGroupRegister>().interaction_groups[self.index.value()]
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
    /// Default value is `0.0`.
    pub restitution: f32,
    /// Friction coefficient of the collision.
    ///
    /// A coefficient of `0.0` means there is no friction (i.e. objects slide completely over each
    /// other).
    ///
    /// Default value is `0.5`.
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
