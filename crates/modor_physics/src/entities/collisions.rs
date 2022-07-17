use crate::colliders::ShapeCollider;
use crate::entities::collisions::internal::UpdateCollidersAction;
use crate::{Collider, Collision, CollisionGroup, Transform};
use modor::{Built, Entity, EntityBuilder, Query};
use modor_internal::ti_vec::TiVecSafeOperations;
use modor_math::Vec3;
use typed_index_collections::TiVec;

idx_type!(pub(crate) GroupIdx);

pub(crate) struct Collisions {
    relationships: TiVec<GroupIdx, TiVec<GroupIdx, Option<CollisionGroupRelationship>>>,
}

#[singleton]
impl Collisions {
    pub fn build<G>() -> impl Built<Self>
    where
        G: CollisionGroup,
    {
        let mut relationships = TiVec::<GroupIdx, TiVec<_, _>>::new();
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
                    let relationship: &mut Option<CollisionGroupRelationship> = relationships
                        .get_mut_or_create(group1_idx)
                        .get_mut_or_create(group2_idx);
                    if let Some(relationship) = relationship {
                        relationship.ignore_z = relationship.ignore_z || layer.ignore_z;
                    } else {
                        relationship.replace(CollisionGroupRelationship {
                            ignore_z: layer.ignore_z,
                        });
                    }
                    let relationship: &mut Option<CollisionGroupRelationship> = relationships
                        .get_mut_or_create(group2_idx)
                        .get_mut_or_create(group1_idx);
                    if let Some(relationship) = relationship {
                        relationship.ignore_z = relationship.ignore_z || layer.ignore_z;
                    } else {
                        relationship.replace(CollisionGroupRelationship {
                            ignore_z: layer.ignore_z,
                        });
                    }
                }
            }
        }
        println!("{relationships:?}");
        EntityBuilder::new(Self { relationships })
    }

    #[run_as(UpdateCollidersAction)]
    fn update_colliders(
        &self,
        mut entities: Query<'_, (&mut Collider, &mut Transform, Entity<'_>)>,
    ) {
        let mut collisions = vec![];
        for (collider1, _, _) in entities.iter_mut() {
            collider1.collisions.clear();
        }
        for (collider1, transform1, entity1) in entities.iter().rev() {
            for (collider2, transform2, entity2) in entities.iter() {
                if entity1.id() == entity2.id() {
                    break;
                }
                if let Some(Some(relationship)) = self
                    .relationships
                    .get(collider1.group_idx)
                    .and_then(|r| r.get(collider2.group_idx))
                {
                    // TODO: use circle collider ?
                    let (position1, position2) = if relationship.ignore_z {
                        (
                            Vec3::xy(transform1.position.x, transform1.position.y),
                            Vec3::xy(transform2.position.x, transform2.position.y),
                        )
                    } else {
                        (transform1.position, transform2.position)
                    };
                    if position1.distance(position2)
                        > transform1.size.magnitude() + transform2.size.magnitude()
                    {
                        continue;
                    }
                    let object1 = ShapeCollider::new(collider1, transform1, relationship);
                    let object2 = ShapeCollider::new(collider2, transform2, relationship);
                    if let Some(collision) = object1.check_collision(&object2) {
                        collisions.push(Collision {
                            entity_id: entity1.id(),
                            other_entity_id: entity2.id(),
                            other_entity_group_idx: collider2.group_idx,
                            normal: collision.penetration_depth,
                        });
                        collisions.push(Collision {
                            entity_id: entity2.id(),
                            other_entity_id: entity1.id(),
                            other_entity_group_idx: collider1.group_idx,
                            normal: collision.penetration_depth * -1.,
                        });
                    }
                }
            }
        }
        for collision in collisions {
            if let Some((collider, _, _)) = entities.get_mut(collision.entity_id) {
                collider.collisions.push(collision);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct CollisionGroupRelationship {
    pub(crate) ignore_z: bool,
}

pub(crate) mod internal {
    use crate::internal::UpdateTransformsFromRelativeAction;

    #[action(UpdateTransformsFromRelativeAction)]
    pub struct UpdateCollidersAction;
}
