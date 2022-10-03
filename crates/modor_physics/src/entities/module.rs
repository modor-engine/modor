use crate::internal::{Update2DAbsoluteFromRelativeAction, Update2DBodies};
use crate::storages_2d::core::{Core2DStorage, PhysicsEntity2DTuple};
use crate::{
    CollisionGroupIndex, CollisionLayer, DeltaTime, RelativeTransform2D, Transform2D,
    ROOT_TRANSFORM,
};
use modor::{Built, Entity, EntityBuilder, Query, Single, With};
use std::time::Duration;

/// The main entity of the physics module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
///
/// # Examples
///
/// ```rust
/// # use std::f32::consts::PI;
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_math::Vec2;
/// # use modor_physics::{
/// #     Transform2D, PhysicsModule, Dynamics2D, RelativeTransform2D, Collider2D,
/// #     CollisionGroupIndex
/// # };
/// #
/// let mut app = App::new()
///     .with_entity(PhysicsModule::build())
///     .with_entity(Object::build());
/// loop {
///     app.update();
///     # break;
/// }
///
/// struct Object;
///
/// #[entity]
/// impl Object {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(
///                 Transform2D::new()
///                     .with_position(Vec2::new(0.2, 0.3))
///                     .with_size(Vec2::new(0.25, 0.5))
///                     .with_rotation(20_f32.to_radians())
///             )
///             .with(RelativeTransform2D::new().with_rotation(PI / 2.))
///             .with(Dynamics2D::new().with_velocity(Vec2::new(-0.01, 0.02)))
///             .with(Collider2D::rectangle(CollisionGroupIndex::Group0))
///     }
/// }
/// ```
///
/// It is also possible to configure collision layers to define which collision groups can collide
/// together:
/// ```rust
/// # use std::f32::consts::PI;
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_math::Vec2;
/// # use modor_physics::{
/// #     Transform2D, PhysicsModule, Dynamics2D, RelativeTransform2D, CollisionGroupIndex,
/// #     CollisionLayer, Collider2D
/// # };
/// #
/// enum CollisionGroup {
///     Ally,
///     Enemy,
///     AllyBullet,
///     EnemyBullet
/// }
///
/// impl From<CollisionGroup> for CollisionGroupIndex {
///     fn from(group: CollisionGroup) -> Self {
///         match group {
///             CollisionGroup::Ally => Self::Group0,
///             CollisionGroup::Enemy => Self::Group1,
///             CollisionGroup::AllyBullet => Self::Group2,
///             CollisionGroup::EnemyBullet => Self::Group2,
///         }
///     }
/// }
///
/// struct Ally;
///
/// #[entity]
/// impl Ally {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Transform2D::new())
///             .with(Collider2D::circle(CollisionGroup::Ally))
///     }
/// }
///
/// let layers = vec![
///     CollisionLayer::new(vec![
///         CollisionGroup::Ally.into(),
///         CollisionGroup::EnemyBullet.into(),
///     ]),
///     CollisionLayer::new(vec![
///         CollisionGroup::Enemy.into(),
///         CollisionGroup::AllyBullet.into(),
///     ]),
/// ];
/// let mut app = App::new()
///     .with_entity(PhysicsModule::build_with_layers(layers))
///     .with_entity(Ally::build());
/// ```
pub struct PhysicsModule {
    groups: Vec<Group>,
    core_2d: Core2DStorage,
}

#[singleton]
impl PhysicsModule {
    /// Builds the module where all entities with a [`Collider2D`](crate::Collider2D) component
    /// can collide with each other.
    pub fn build() -> impl Built<Self> {
        Self::build_with_layers(vec![CollisionLayer::new(
            CollisionGroupIndex::ALL
                .into_iter()
                .chain(CollisionGroupIndex::ALL.into_iter())
                .collect(),
        )])
    }

    /// Builds the module with custom collision layers.
    pub fn build_with_layers(layers: Vec<CollisionLayer>) -> impl Built<Self> {
        EntityBuilder::new(Self {
            groups: Self::compute_groups(layers),
            core_2d: Core2DStorage::default(),
        })
        .with_child(DeltaTime::build(Duration::ZERO))
    }

    #[run_as(Update2DAbsoluteFromRelativeAction)]
    fn update_2d_absolute_from_relative(
        entities: Query<'_, Entity<'_>, (With<Transform2D>, With<RelativeTransform2D>)>,
        mut components: Query<'_, (&mut Transform2D, Option<&mut RelativeTransform2D>)>,
    ) {
        for entity in Self::entities_sorted_by_depth(entities.iter()) {
            match components.get_with_first_parent_mut(entity.id()) {
                (Some((transform, Some(relative))), Some((parent, _))) => {
                    transform.update_from_relative(relative, parent);
                }
                (Some((transform, Some(relative))), None) => {
                    transform.update_from_relative(relative, &ROOT_TRANSFORM);
                }
                _ => unreachable!("internal error: unreachable absolute transform update case"),
            }
        }
    }

    #[run_as(Update2DBodies)]
    fn update_2d_bodies(
        &mut self,
        delta: Single<'_, DeltaTime>,
        mut entities: Query<'_, PhysicsEntity2DTuple<'_>>,
    ) {
        self.core_2d
            .update(delta.get(), &mut entities, &self.groups);
    }

    #[run_as(UpdatePhysicsAction)]
    fn finish_update() {}

    fn compute_groups(layers: Vec<CollisionLayer>) -> Vec<Group> {
        let mut groups: Vec<_> = (0..32).map(Group::new).collect();
        for layer in layers {
            let group_idxs: Vec<_> = layer.groups.into_iter().map(|g| g as usize).collect();
            for (group_pos, &group_idx) in group_idxs.iter().enumerate() {
                for &current_group_idx in &group_idxs[0..group_pos] {
                    groups[current_group_idx].interaction_bits |= 1 << group_idx;
                    groups[group_idx].interaction_bits |= 1 << current_group_idx;
                }
            }
        }
        groups
    }

    fn entities_sorted_by_depth<'a, I>(entities: I) -> Vec<Entity<'a>>
    where
        I: Iterator<Item = Entity<'a>>,
    {
        let mut entities: Vec<_> = entities.collect();
        entities.sort_unstable_by_key(|e| e.depth());
        entities
    }
}

pub(crate) struct Group {
    pub(crate) membership_bits: u32, // the active bit indicates the group
    pub(crate) interaction_bits: u32, // the active bits indicates with which groups it can collide
}

impl Group {
    pub(crate) const fn new(group_idx: usize) -> Self {
        Self {
            membership_bits: 1 << group_idx,
            interaction_bits: 0,
        }
    }
}

/// An action done when the transforms and colliders have been updated.
#[action(Update2DBodies)]
pub struct UpdatePhysicsAction;

pub(crate) mod internal {
    #[action]
    pub struct Update2DAbsoluteFromRelativeAction;

    #[action(Update2DAbsoluteFromRelativeAction)]
    pub struct Update2DBodies;
}
