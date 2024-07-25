use crate::user_data::ColliderUserData;
use crate::{CollisionGroupGlob, CollisionType};
use fxhash::FxHashMap;
use modor::{App, Globals, RootNode};
use rapier2d::geometry::{Group, InteractionGroups, SolverFlags};
use rapier2d::pipeline::{ContactModificationContext, PairFilterContext};

#[derive(Debug, Default)]
pub(crate) struct PhysicsHooks {
    collision_types: FxHashMap<(usize, usize), CollisionType>,
}

impl RootNode for PhysicsHooks {
    fn on_create(_app: &mut App) -> Self {
        Self::default()
    }

    fn update(&mut self, app: &mut App) {
        for &(index, _) in app.get_mut::<Globals<CollisionGroupGlob>>().deleted_items() {
            self.collision_types
                .retain(|&(index1, index2), _| index != index1 && index != index2);
        }
    }
}

impl rapier2d::pipeline::PhysicsHooks for PhysicsHooks {
    fn filter_contact_pair(&self, context: &PairFilterContext<'_>) -> Option<SolverFlags> {
        let group1_index =
            ColliderUserData::from(context.colliders[context.collider1].user_data).group_index();
        let group2_index =
            ColliderUserData::from(context.colliders[context.collider2].user_data).group_index();
        match self.collision_types.get(&(group1_index, group2_index))? {
            CollisionType::Sensor => Some(SolverFlags::empty()),
            CollisionType::Impulse(_) => Some(SolverFlags::COMPUTE_IMPULSES),
        }
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext<'_>) {
        let group1_index =
            ColliderUserData::from(context.colliders[context.collider1].user_data).group_index();
        let group2_index =
            ColliderUserData::from(context.colliders[context.collider2].user_data).group_index();
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
    pub(crate) fn interactions(&self, index: usize) -> InteractionGroups {
        let mut filter = Group::empty();
        for &(index1, index2) in self.collision_types.keys() {
            if index1 == index || index2 == index {
                filter |= Group::from(1 << (index2 % 32));
                filter |= Group::from(1 << (index1 % 32));
            }
        }
        InteractionGroups::new(Group::from(1 << (index % 32)), filter)
    }

    pub(crate) fn add_interaction(&mut self, index1: usize, index2: usize, type_: CollisionType) {
        self.collision_types.insert((index1, index2), type_);
        self.collision_types.insert((index2, index1), type_);
    }
}
