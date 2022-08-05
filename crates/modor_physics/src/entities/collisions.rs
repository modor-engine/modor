use crate::entities::collisions::internal::{CheckCollisionsAction, UpdateCollidersAction};
use crate::{Collider, Collision, CollisionGroup, Transform};
use modor::{Built, Entity, EntityBuilder, Query};
use modor_internal::ti_vec::TiVecSafeOperations;
use typed_index_collections::TiVec;

idx_type!(pub(crate) GroupIdx);

pub(crate) struct Collisions {
    can_collide: TiVec<GroupIdx, TiVec<GroupIdx, bool>>,
}

#[singleton]
impl Collisions {
    pub fn build<G>() -> impl Built<Self>
    where
        G: CollisionGroup,
    {
        let mut groups_can_collide = TiVec::<GroupIdx, TiVec<_, _>>::new();
        for layer in G::layers() {
            let groups: Vec<_> = layer
                .groups
                .into_iter()
                .map(|g| g.index())
                .map(Into::into)
                .collect();
            for (index1, &group1_idx) in groups.iter().enumerate().rev() {
                for (index2, &group2_idx) in groups.iter().enumerate() {
                    if index1 == index2 {
                        break;
                    }
                    *groups_can_collide
                        .get_mut_or_create(group1_idx)
                        .get_mut_or_create(group2_idx) = true;
                    *groups_can_collide
                        .get_mut_or_create(group2_idx)
                        .get_mut_or_create(group1_idx) = true;
                }
            }
        }
        EntityBuilder::new(Self {
            can_collide: groups_can_collide,
        })
    }

    #[run_as(UpdateCollidersAction)]
    fn update_colliders(&self, mut entities: Query<'_, (&mut Collider, &Transform)>) {
        for (collider, transform) in entities.iter_mut() {
            collider.update(transform);
        }
    }

    #[run_as(CheckCollisionsAction)]
    fn check_collisions(&self, mut entities: Query<'_, (&mut Collider, Entity<'_>)>) {
        let mut collisions = vec![];
        for (collider1, _) in entities.iter_mut() {
            collider1.collisions.clear();
        }
        for (collider1, entity1) in entities.iter().rev() {
            for (collider2, entity2) in entities.iter() {
                if entity1.id() == entity2.id() {
                    break;
                }
                if let Some(true) = self
                    .can_collide
                    .get(collider1.group_idx)
                    .and_then(|r| r.get(collider2.group_idx))
                {
                    if !collider1
                        .simplified_shape
                        .is_colliding_with(&collider2.simplified_shape)
                    {
                        continue;
                    }
                    if let Some(collision) = collider1.shape.check_collision(&collider2.shape) {
                        collisions.push(Collision {
                            entity_id: entity1.id(),
                            other_entity_id: entity2.id(),
                            other_entity_group_idx: collider2.group_idx,
                            penetration: collision.penetration,
                            contact_centroid: collision.contact_centroid,
                        });
                        collisions.push(Collision {
                            entity_id: entity2.id(),
                            other_entity_id: entity1.id(),
                            other_entity_group_idx: collider1.group_idx,
                            penetration: collision.penetration * -1.,
                            contact_centroid: collision.contact_centroid,
                        });
                    }
                }
            }
        }
        for collision in collisions {
            if let Some((collider, _)) = entities.get_mut(collision.entity_id) {
                collider.collisions.push(collision);
            }
        }
    }
}

pub(crate) mod internal {
    use crate::internal::UpdateTransformsFromRelativeAction;

    #[action(UpdateTransformsFromRelativeAction)]
    pub struct UpdateCollidersAction;

    #[action(UpdateCollidersAction)]
    pub struct CheckCollisionsAction;
}
