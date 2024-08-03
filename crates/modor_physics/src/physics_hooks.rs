use crate::user_data::ColliderUserData;
use crate::{CollisionGroup, Impulse};
use fxhash::FxHashMap;
use modor::{App, FromApp, Globals, State};
use rapier2d::geometry::{ColliderHandle, ColliderSet, Group, InteractionGroups, SolverFlags};
use rapier2d::pipeline::{ContactModificationContext, PairFilterContext};

#[derive(Debug, FromApp)]
pub(crate) struct PhysicsHooks {
    pub(crate) interaction_groups: Vec<InteractionGroups>,
    collision_types: FxHashMap<(usize, usize), CollisionType>,
}

impl State for PhysicsHooks {
    fn update(&mut self, app: &mut App) {
        for &(index, _) in app.get_mut::<Globals<CollisionGroup>>().deleted_items() {
            self.interaction_groups[index] = Self::default_group(index);
            self.collision_types
                .retain(|&(index1, index2), _| index != index1 && index != index2);
        }
    }
}

impl rapier2d::pipeline::PhysicsHooks for PhysicsHooks {
    fn filter_contact_pair(&self, context: &PairFilterContext<'_>) -> Option<SolverFlags> {
        let group1_index = Self::group_index(context.colliders, context.collider1);
        let group2_index = Self::group_index(context.colliders, context.collider2);
        match self.collision_types.get(&(group1_index, group2_index))? {
            CollisionType::Sensor => Some(SolverFlags::empty()),
            CollisionType::Impulse(_) => Some(SolverFlags::COMPUTE_IMPULSES),
        }
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext<'_>) {
        let group1_index = Self::group_index(context.colliders, context.collider1);
        let group2_index = Self::group_index(context.colliders, context.collider2);
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

impl PhysicsHooks {
    pub(crate) fn register_group(&mut self, index: usize) {
        for index in self.interaction_groups.len()..=index {
            self.interaction_groups.push(Self::default_group(index));
        }
    }

    pub(crate) fn add_interaction(&mut self, index1: usize, index2: usize, type_: CollisionType) {
        self.interaction_groups[index1].filter |= Group::from(1 << (index2 % 32));
        self.interaction_groups[index2].filter |= Group::from(1 << (index1 % 32));
        self.collision_types.insert((index1, index2), type_);
        self.collision_types.insert((index2, index1), type_);
    }

    fn default_group(index: usize) -> InteractionGroups {
        InteractionGroups::new(Group::from(1 << (index % 32)), Group::empty())
    }

    fn group_index(colliders: &ColliderSet, collider: ColliderHandle) -> usize {
        ColliderUserData::from(colliders[collider].user_data).group_index()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[non_exhaustive]
pub(crate) enum CollisionType {
    /// Collision should happen but it doesn't produce forces.
    Sensor,
    /// Collision should happen and it produces forces.
    ///
    /// Note that there is no effect if the body is not dynamic, or if its mass and angular inertia
    /// are equal to zero.
    Impulse(Impulse),
}
