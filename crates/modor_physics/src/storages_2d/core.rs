use super::collision_groups::CollisionGroupStorage;
use crate::storages_2d::bodies::BodyStorage;
use crate::storages_2d::colliders::ColliderStorage;
use crate::storages_2d::pipeline::PipelineStorage;
use crate::utils::UserData;
use crate::{Collider2D, Collision2D, Dynamics2D, RelativeTransform2D, Transform2D};
use modor::{Entity, Filter, Or, Query, With};
use rapier2d::dynamics::RigidBody;
use rapier2d::geometry::Collider;
use rapier2d::prelude::InteractionGroups;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct Core2DStorage {
    bodies: BodyStorage,
    colliders: ColliderStorage,
    collision_groups: CollisionGroupStorage,
    pipeline: PipelineStorage,
}

impl Core2DStorage {
    pub(crate) fn update(
        &mut self,
        delta: Duration,
        entities: &mut Query<'_, PhysicsEntity2DTuple<'_>>,
    ) {
        for entity in entities.iter_mut() {
            let mut entity = PhysicsEntity2D::from(entity);
            self.bodies.delete_outdated_handles(&mut entity);
            self.colliders.delete_outdated_handles(&mut entity);
        }
        self.bodies
            .delete_outdated(entities, &mut self.colliders, &mut self.pipeline);
        self.colliders
            .delete_outdated(entities, &mut self.bodies, &mut self.pipeline);
        for entity in entities.iter_mut() {
            let mut entity = PhysicsEntity2D::from(entity);
            self.register_collision_groups(&mut entity);
        }
        for entity in entities.iter_mut() {
            let mut entity = PhysicsEntity2D::from(entity);
            self.create_resources(&mut entity);
            self.update_resources(&mut entity);
        }
        for entity in entities.iter_mut() {
            let mut entity = PhysicsEntity2D::from(entity);
            self.update_resources(&mut entity);
        }
        self.run_pipeline_step(delta);
        for entity in entities.iter_mut() {
            self.update_entity(&mut entity.into());
        }
        self.update_entity_colliders(entities);
    }

    fn register_collision_groups(&mut self, entity: &mut PhysicsEntity2D<'_>) {
        if let Some(collider) = entity.collider.as_mut() {
            collider.group_idx = Some(if let Some(group_idx) = collider.group_idx {
                group_idx
            } else {
                let group_idx = self.collision_groups.register(&collider.group_key);
                collider.group_idx = Some(group_idx);
                group_idx
            });
        }
    }

    fn create_resources(&mut self, entity: &mut PhysicsEntity2D<'_>) {
        let body_state = self.bodies.create(entity);
        self.colliders
            .create(entity, body_state, &mut self.bodies, &mut self.pipeline);
    }

    fn update_resources(&mut self, entity: &mut PhysicsEntity2D<'_>) {
        let mut body = entity.body_mut(&mut self.bodies);
        let mut collider = entity.collider_mut(&mut self.colliders);
        entity
            .transform
            .update_resources(&mut body, &mut collider, entity.collider.as_ref());
        if let (Some(dynamics), Some(body)) = (&mut entity.dynamics, body) {
            dynamics.update_body(body);
        }
        if let (Some(collider), Some(rapier_collider)) = (&mut entity.collider, collider) {
            let group_idx = collider
                .group_idx
                .expect("internal error: missing collider group index");
            rapier_collider.set_collision_groups(InteractionGroups::new(
                group_idx.group_membership(),
                self.collision_groups.group_filter(group_idx),
            ));
            rapier_collider.user_data = UserData::from(rapier_collider.user_data)
                .with_collision_group_idx(group_idx)
                .into();
        }
    }

    fn run_pipeline_step(&mut self, delta: Duration) {
        self.pipeline.run_pipeline_step(
            delta,
            &mut self.bodies,
            &mut self.colliders,
            &self.collision_groups,
        );
    }

    fn update_entity(&self, entity: &mut PhysicsEntity2D<'_>) {
        if !entity.is_relative {
            if let Some(dynamics) = &mut entity.dynamics {
                let handle = dynamics
                    .handle
                    .expect("internal error: rigid body not registered");
                let body = &self
                    .bodies
                    .get(handle)
                    .expect("internal error: missing rapier body");
                entity.transform.update_from_body(body);
                dynamics.update_from_body(body);
            }
        }
        if let Some(collider) = &mut entity.collider {
            collider.collisions.clear();
        }
    }

    fn update_entity_colliders(&self, entities: &mut Query<'_, PhysicsEntity2DTuple<'_>>) {
        for contact in self.pipeline.contacts(&self.colliders) {
            let entity1_id = contact.entity1_id;
            let entity2_id = contact.entity2_id;
            let (entity1, entity2) = entities.get_both_mut(entity1_id, entity2_id);
            let entity1 = entity1.expect("internal error: missing collider 1 entity");
            let entity2 = entity2.expect("internal error: missing collider 2 entity");
            let entity1 = PhysicsEntity2D::from(entity1);
            let entity2 = PhysicsEntity2D::from(entity2);
            let collider1 = entity1
                .collider
                .expect("internal error: collider 1 not registered");
            let collider2 = entity2
                .collider
                .expect("internal error: collider 2 not registered");
            for manifold in contact.manifolds {
                if !manifold.points.is_empty() {
                    let (collision1, collision2) = Collision2D::create_pair(
                        entity1_id,
                        entity2_id,
                        collider1.group_key.clone(),
                        collider2.group_key.clone(),
                        entity1.transform,
                        entity2.transform,
                        manifold,
                    );
                    collider1.collisions.push(collision1);
                    collider2.collisions.push(collision2);
                }
            }
        }
    }
}

pub(crate) type PhysicsEntity2DTuple<'a> = (
    Entity<'a>,
    &'a mut Transform2D,
    Option<&'a mut Dynamics2D>,
    Option<&'a mut Collider2D>,
    Option<&'a mut RelativeTransform2D>,
    Filter<Or<(With<Dynamics2D>, With<Collider2D>)>>,
);

pub(super) struct PhysicsEntity2D<'a> {
    pub(super) entity: Entity<'a>,
    pub(super) transform: &'a mut Transform2D,
    pub(super) dynamics: Option<&'a mut Dynamics2D>,
    pub(super) collider: Option<&'a mut Collider2D>,
    pub(super) is_relative: bool,
}

impl PhysicsEntity2D<'_> {
    fn body_mut<'a>(&mut self, bodies: &'a mut BodyStorage) -> Option<&'a mut RigidBody> {
        if self.is_relative {
            return None;
        }
        self.dynamics
            .as_mut()
            .and_then(|d| d.handle)
            .and_then(|h| bodies.get_mut(h))
    }

    fn collider_mut<'a>(&mut self, colliders: &'a mut ColliderStorage) -> Option<&'a mut Collider> {
        self.collider
            .as_mut()
            .and_then(|c| c.handle)
            .and_then(|h| colliders.get_mut(h))
    }
}

impl<'a> From<PhysicsEntity2DTuple<'a>> for PhysicsEntity2D<'a> {
    fn from(tuple: PhysicsEntity2DTuple<'a>) -> Self {
        let (entity, transform, dynamics, collider, relative, _) = tuple;
        Self {
            entity,
            transform,
            dynamics,
            collider,
            is_relative: relative.is_some(),
        }
    }
}
