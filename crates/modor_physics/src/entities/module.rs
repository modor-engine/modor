use crate::storages_2d::core::{Core2DStorage, PhysicsEntity2DTuple};
use crate::{DeltaTime, RelativeTransform2D, Transform2D, ROOT_TRANSFORM};
use modor::{BuiltEntity, Entity, EntityBuilder, Filter, Query, Single, With};
use std::time::Duration;

type RelativeTransform2DFilter = Filter<(With<Transform2D>, With<RelativeTransform2D>)>;

/// The main entity of the physics module.
///
/// # Examples
///
/// ```rust
/// # use std::f32::consts::PI;
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// let mut app = App::new()
///     .with_entity(PhysicsModule::build())
///     .with_entity(build_object());
/// loop {
///     app.update();
///     # break;
/// }
///
/// fn build_object() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .with(|t| *t.position = Vec2::new(0.2, 0.3))
///         .with(|t| *t.size = Vec2::new(0.25, 0.5))
///         .with(|t| *t.rotation = 20_f32.to_radians())
///         .component(RelativeTransform2D::new())
///         .with(|t| *t.rotation = PI / 2.)
///         .component(Dynamics2D::new())
///         .with(|d| *d.velocity = Vec2::new(-0.01, 0.02))
/// }
/// ```
///
/// Colliders can be configured this way:
/// ```rust
/// # use std::f32::consts::PI;
/// # use modor::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// let mut app = App::new()
///     .with_entity(PhysicsModule::build())
///     .with_entity(Ally::build());
///
/// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// enum CollisionGroup {
///     Ally,
///     Enemy,
///     AllyBullet,
///     EnemyBullet
/// }
///
/// impl CollisionGroupRef for CollisionGroup {
///     fn collision_type(&self, other: &Self) -> CollisionType {
///         match (self, other) {
///             (Self::Ally, Self::EnemyBullet) => CollisionType::Sensor,
///             (Self::Enemy, Self::AllyBullet) => CollisionType::Sensor,
///             _ => CollisionType::None,
///         }
///     }
/// }
///
/// #[derive(Component)]
/// struct Ally;
///
/// #[systems]
/// impl Ally {
///     fn build() -> impl BuiltEntity {
///         EntityBuilder::new()
///             .component(Self)
///             .component(Transform2D::new())
///             .component(Collider2D::circle(CollisionGroup::Ally))
///     }
/// }
/// ```
#[derive(SingletonComponent)]
pub struct PhysicsModule {
    core_2d: Core2DStorage,
}

#[systems]
impl PhysicsModule {
    /// Builds the module where all entities with a [`Collider2D`](crate::Collider2D) component
    /// can collide with each other.
    pub fn build() -> impl BuiltEntity {
        info!("physics module created");
        EntityBuilder::new()
            .component(Self {
                core_2d: Core2DStorage::default(),
            })
            .child_component(DeltaTime::from(Duration::ZERO))
    }

    #[run]
    fn update_2d_absolute_from_relative(
        entities: Query<'_, (Entity<'_>, RelativeTransform2DFilter)>,
        mut components: Query<'_, (&mut Transform2D, Option<&mut RelativeTransform2D>)>,
    ) {
        for entity in Self::entities_sorted_by_depth(entities.iter().map(|(e, _)| e)) {
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

    #[run_after_previous]
    fn update_2d_bodies(
        &mut self,
        delta: Single<'_, DeltaTime>,
        mut entities: Query<'_, PhysicsEntity2DTuple<'_>>,
    ) {
        self.core_2d.update(delta.get(), &mut entities);
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
