use crate::components::collider::ColliderUserData;
use crate::{CollisionGroup, CollisionType, Impulse};
use fxhash::FxHashMap;
use modor::Query;
use modor_resources::Resource;
use rapier2d::geometry::SolverFlags;
use rapier2d::pipeline::{ContactModificationContext, PairFilterContext};

#[derive(SingletonComponent, Default)]
pub(crate) struct PhysicsHook {
    collision_types: FxHashMap<u64, FxHashMap<u64, CollisionType>>,
    collision_impulses: FxHashMap<u64, FxHashMap<u64, Impulse>>,
}

#[systems]
impl PhysicsHook {
    #[run_after(component(CollisionGroup))]
    fn update(&mut self, groups: Query<'_, &CollisionGroup>) {
        self.collision_types.clear();
        self.collision_impulses.clear();
        for group1 in groups.iter() {
            for group2 in groups.iter() {
                let type1 = (group1.collision_type_fn)(group2.key());
                let type2 = (group2.collision_type_fn)(group1.key());
                let collision_type = type1.highest_priority(type2);
                *self
                    .collision_types
                    .entry(group1.id)
                    .or_default()
                    .entry(group2.id)
                    .or_default() = collision_type;
                if let CollisionType::Impulse(impulse) = collision_type {
                    *self
                        .collision_impulses
                        .entry(group1.id)
                        .or_default()
                        .entry(group2.id)
                        .or_default() = impulse;
                }
            }
        }
    }
}

impl rapier2d::pipeline::PhysicsHooks for PhysicsHook {
    fn filter_contact_pair(&self, context: &PairFilterContext<'_>) -> Option<SolverFlags> {
        let data1: ColliderUserData = context.colliders[context.collider1].user_data.into();
        let data2: ColliderUserData = context.colliders[context.collider2].user_data.into();
        let collision_type = self
            .collision_types
            .get(&data1.group_id())
            .and_then(|types| types.get(&data2.group_id()))
            .copied()
            .unwrap_or_default();
        match collision_type {
            CollisionType::None => None,
            CollisionType::Sensor => Some(SolverFlags::empty()),
            CollisionType::Impulse(_) => Some(SolverFlags::COMPUTE_IMPULSES),
        }
    }

    fn modify_solver_contacts(&self, context: &mut ContactModificationContext<'_>) {
        let data1: ColliderUserData = context.colliders[context.collider1].user_data.into();
        let data2: ColliderUserData = context.colliders[context.collider2].user_data.into();
        if let Some(impulse) = self
            .collision_impulses
            .get(&data1.group_id())
            .and_then(|types| types.get(&data2.group_id()))
            .copied()
        {
            for contact in context.solver_contacts.iter_mut() {
                contact.restitution = impulse.restitution;
                contact.friction = impulse.friction;
            }
        }
    }
}
