use crate::storages_2d::bodies::BodyStorage;
use crate::storages_2d::colliders::ColliderStorage;
use crate::storages_2d::pipeline::PipelineStorage;
use crate::{Collider2D, Collision2D, Dynamics2D, Group, RelativeTransform2D, Transform2D};
use modor::{Entity, Query};
use rapier2d::dynamics::RigidBody;
use rapier2d::geometry::Collider;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct Core2DStorage {
    bodies: BodyStorage,
    colliders: ColliderStorage,
    pipeline: PipelineStorage,
}

impl Core2DStorage {
    pub(crate) fn update(
        &mut self,
        delta: Duration,
        entities: &mut Query<'_, PhysicsEntity2DTuple<'_>>,
        groups: &[Group],
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
            self.create_resources(&mut entity, groups);
            self.update_resources(&mut entity);
        }
        self.run_pipeline_step(delta);
        for entity in entities.iter_mut() {
            self.update_entity(&mut entity.into());
        }
        self.update_entity_colliders(entities);
    }

    fn create_resources(&mut self, entity: &mut PhysicsEntity2D<'_>, groups: &[Group]) {
        let body_state = self.bodies.create(entity);
        self.colliders.create(
            entity,
            body_state,
            &mut self.bodies,
            &mut self.pipeline,
            groups,
        );
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
    }

    fn run_pipeline_step(&mut self, delta: Duration) {
        self.pipeline
            .run_pipeline_step(delta, &mut self.bodies, &mut self.colliders);
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
            let entity1 = entity1.expect("internal error: missing collider1 entity");
            let entity2 = entity2.expect("internal error: missing collider2 entity");
            let entity1 = PhysicsEntity2D::from(entity1);
            let entity2 = PhysicsEntity2D::from(entity2);
            let collider1 = entity1
                .collider
                .expect("internal error: collider1 not registered");
            let collider2 = entity2
                .collider
                .expect("internal error: collider1 not registered");
            for manifold in contact.manifolds {
                if !manifold.points.is_empty() {
                    let (collision1, collision2) = Collision2D::create_pair(
                        entity1_id,
                        entity2_id,
                        collider1.group_idx,
                        collider2.group_idx,
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
        let (entity, transform, dynamics, collider, relative) = tuple;
        Self {
            entity,
            transform,
            dynamics,
            collider,
            is_relative: relative.is_some(),
        }
    }
}
