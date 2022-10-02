pub mod collider_2d;
pub mod dynamics_2d;
pub mod relative_transform_2d;
pub mod transform_2d;

use modor::{Built, Entity, EntityBuilder, Query, With, World};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupIndex, Dynamics2D, RelativeTransform2D, Transform2D,
    UpdatePhysicsAction,
};
use std::mem;
use std::time::Duration;

const DELTA_TIME: Duration = Duration::from_secs(2);

struct MoveDst;

enum Action {
    SetPosition(Vec2),
    SetSize(Vec2),
    SetRotation(f32),
    SetRelativePosition(Option<Vec2>),
    SetRelativeSize(Option<Vec2>),
    SetRelativeRotation(Option<f32>),
    SetVelocity(Vec2),
    SetAngularVelocity(f32),
    RemoveDynamics,
    MoveDynamics,
    PutBackDynamics,
    RemoveCollider,
    MoveCollider,
    PutBackCollider,
}

struct Updates {
    actions: Vec<Action>,
    dynamics: Option<Dynamics2D>,
    collider: Option<Collider2D>,
}

#[singleton]
impl Updates {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            actions: vec![],
            dynamics: None,
            collider: None,
        })
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[run_after(UpdatePhysicsAction)]
    fn update_transform(&mut self, transform: &mut Transform2D) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                Action::SetPosition(p) => {
                    *transform.position = p;
                    None
                }
                Action::SetSize(s) => {
                    *transform.size = s;
                    None
                }
                Action::SetRotation(r) => {
                    *transform.rotation = r;
                    None
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[run_after_previous]
    fn update_relative_transform(&mut self, transform: &mut RelativeTransform2D) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                Action::SetRelativePosition(p) => {
                    transform.position = p;
                    None
                }
                Action::SetRelativeSize(s) => {
                    transform.size = s;
                    None
                }
                Action::SetRelativeRotation(r) => {
                    transform.rotation = r;
                    None
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[run_after_previous]
    fn update_dynamics(&mut self, dynamics: &mut Dynamics2D) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                action @ Action::RemoveDynamics => {
                    self.dynamics = Some(mem::replace(dynamics, Dynamics2D::new()));
                    Some(action)
                }
                Action::SetVelocity(v) => {
                    *dynamics.velocity = v;
                    None
                }
                Action::SetAngularVelocity(v) => {
                    *dynamics.angular_velocity = v;
                    None
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }

    #[allow(clippy::wildcard_enum_match_arm, clippy::unnecessary_filter_map)]
    #[run_after_previous]
    fn update_collider(&mut self, collider: &mut Collider2D) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                action @ Action::RemoveCollider => {
                    self.collider = Some(mem::replace(
                        collider,
                        Collider2D::rectangle(CollisionGroupIndex::Group0),
                    ));
                    Some(action)
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[run_after_previous]
    fn update_entity(&mut self, entity: Entity<'_>, mut world: World<'_>) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                Action::RemoveDynamics => {
                    world.delete_component::<Dynamics2D>(entity.id());
                    None
                }
                Action::RemoveCollider => {
                    world.delete_component::<Collider2D>(entity.id());
                    None
                }
                Action::PutBackDynamics => {
                    world.add_component(entity.id(), self.dynamics.take().unwrap());
                    None
                }
                Action::PutBackCollider => {
                    world.add_component(entity.id(), self.collider.take().unwrap());
                    None
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[run_after_previous]
    fn move_dynamics(
        &mut self,
        dynamics: &mut Dynamics2D,
        mut world: World<'_>,
        move_dest_entities: Query<'_, Entity<'_>, With<MoveDst>>,
    ) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                Action::MoveDynamics => {
                    let dynamics = mem::replace(dynamics, Dynamics2D::new());
                    let dest_entity_id = move_dest_entities.iter().next().unwrap().id();
                    world.add_component(dest_entity_id, dynamics);
                    None
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    #[run_after_previous]
    fn move_collider(
        &mut self,
        collider: &mut Collider2D,
        mut world: World<'_>,
        move_dest_entities: Query<'_, Entity<'_>, With<MoveDst>>,
    ) {
        let remaining_actions = self
            .actions
            .drain(..)
            .filter_map(|a| match a {
                Action::MoveCollider => {
                    let collider =
                        mem::replace(collider, Collider2D::rectangle(CollisionGroupIndex::Group0));
                    let dest_entity_id = move_dest_entities.iter().next().unwrap().id();
                    world.add_component(dest_entity_id, collider);
                    None
                }
                a => Some(a),
            })
            .collect();
        self.actions = remaining_actions;
    }
}

struct TestEntity;

#[entity]
impl TestEntity {}
