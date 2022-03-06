use crate::module::internal::{
    UpdateAbsolutePositionsAction, UpdateAbsoluteScalesAction, UpdatePositionsAction,
    UpdateVelocitiesAction,
};
use crate::{Acceleration, DeltaTime, Position, Scale, Velocity};
use modor::{
    system, Action, Built, DependsOn, Entity, EntityBuilder, EntityMainComponent, Query, Single,
    Singleton, SystemRunner, With,
};

/// The main entity of the physics module.
///
/// # Examples
///
/// ```rust
/// # use modor::{App, Built, EntityBuilder, EntityMainComponent};
/// # use modor_physics::{Acceleration, PhysicsModule, Position, Scale, Shape, Velocity};
///
/// let mut app = App::new()
///     .with_entity::<PhysicsModule>(())
///     .with_entity::<Object>(());
/// loop {
///     app.update();
///     # break;
/// }
///
/// struct Object;
///
/// impl EntityMainComponent for Object {
///     type Type = ();
///     type Data = ();
///
///     fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built<'_> {
///         builder
///             .with(Position::xy(0.2, 0.3))
///             .with(Velocity::xy(-0.01, 0.02))
///             .with(Acceleration::xy(0.5, -0.1))
///             .with(Scale::xy(0.25, 0.5))
///             .with(Shape::Rectangle2D)
///             .with_self(Self)
///     }
/// }
/// ```
pub struct PhysicsModule;

impl EntityMainComponent for PhysicsModule {
    type Type = Singleton;
    type Data = ();

    fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built<'_> {
        builder.with_child::<DeltaTime>(()).with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
            .run_as::<UpdateVelocitiesAction>(system!(Self::update_velocities))
            .run_as::<UpdatePositionsAction>(system!(Self::update_positions))
            .run_as::<UpdateAbsoluteScalesAction>(system!(Self::update_absolute_scales))
            .run_as::<UpdateAbsolutePositionsAction>(system!(Self::update_absolute_positions))
            .run_as::<PhysicsUpdateAction>(system!(|| ()))
    }
}

impl PhysicsModule {
    fn update_velocities(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut Velocity, &Acceleration)>,
    ) {
        for (velocity, acceleration) in components.iter_mut() {
            velocity.update(acceleration, delta_time.get());
        }
    }

    fn update_positions(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut Position, &Velocity)>,
    ) {
        for (position, velocity) in components.iter_mut() {
            position.update(velocity, delta_time.get());
        }
    }

    fn update_absolute_scales(
        entities_with_scale: Query<'_, Entity<'_>, (With<Position>, With<Scale>)>,
        mut scales: Query<'_, &mut Scale, With<Position>>,
    ) {
        let entities = Self::sorted_by_depth(entities_with_scale.iter());
        for entity in entities {
            let result = scales.get_with_first_parent_mut(entity.id());
            if let (Some(scale), Some(parent_scale)) = result {
                scale.update_abs(parent_scale);
            }
        }
    }

    fn update_absolute_positions(
        entities_with_position: Query<'_, Entity<'_>, With<Position>>,
        mut components: Query<'_, (&mut Position, Option<&mut Scale>)>,
    ) {
        let entities = Self::sorted_by_depth(entities_with_position.iter());
        for entity in entities {
            let result = components.get_with_first_parent_mut(entity.id());
            if let (Some((position, _)), Some((parent_position, parent_scale))) = result {
                position.update_abs(parent_position, parent_scale.as_deref());
            }
        }
    }

    fn sorted_by_depth<'a, I>(entities: I) -> Vec<Entity<'a>>
    where
        I: Iterator<Item = Entity<'a>>,
    {
        let mut entities: Vec<_> = entities.collect();
        entities.sort_unstable_by_key(|e| e.depth());
        entities
    }
}

/// An action done when the positions and scales have been updated.
pub struct PhysicsUpdateAction;

impl Action for PhysicsUpdateAction {
    type Constraint = DependsOn<UpdateAbsolutePositionsAction>;
}

mod internal {
    use crate::UpdateDeltaTimeAction;
    use modor::define_action;

    define_action!(UpdateVelocitiesAction: UpdateDeltaTimeAction, pub);
    define_action!(UpdatePositionsAction: UpdateVelocitiesAction, pub);
    define_action!(UpdateAbsoluteScalesAction: UpdateDeltaTimeAction, pub);
    define_action!(
        UpdateAbsolutePositionsAction: UpdateAbsoluteScalesAction,
        pub
    );
}

#[cfg(test)]
mod physics_module_tests {
    use crate::{Acceleration, DeltaTime, PhysicsModule, Position, Scale, Velocity};
    use approx::assert_abs_diff_eq;
    use modor::testing::{TestApp, TestEntity};
    use modor::App;

    #[test]
    fn build() {
        let app: TestApp = App::new().with_entity::<PhysicsModule>(()).into();
        app.assert_singleton::<PhysicsModule>().exists();
        app.assert_singleton::<DeltaTime>().exists();
    }

    #[test]
    fn update_velocity_and_position() {
        let mut app: TestApp = App::new().with_entity::<PhysicsModule>(()).into();
        let entity_id = app.create_entity::<TestEntity<_>>(Box::new(|b| {
            b.with(Position::xyz(1., 2., 3.))
                .with(Velocity::xyz(4., 5., 6.))
                .with(Acceleration::xyz(7., 8., 9.))
        }));
        app.update();
        let mut delta_time = 0.;
        app.assert_singleton::<DeltaTime>()
            .has::<DeltaTime, _>(|d| delta_time = d.get().as_secs_f32());
        app.assert_entity(entity_id)
            .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.x, 7.))
            .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.y, 8.))
            .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.z, 9.))
            .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.x, 7.0_f32.mul_add(delta_time, 4.)))
            .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.y, 8.0_f32.mul_add(delta_time, 5.)))
            .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.z, 9.0_f32.mul_add(delta_time, 6.)))
            .has::<Position, _>(|p| {
                assert_abs_diff_eq!(p.x, 7.0_f32.mul_add(delta_time, 4.).mul_add(delta_time, 1.));
            })
            .has::<Position, _>(|p| {
                assert_abs_diff_eq!(p.y, 8.0_f32.mul_add(delta_time, 5.).mul_add(delta_time, 2.));
            })
            .has::<Position, _>(|p| {
                assert_abs_diff_eq!(p.z, 9.0_f32.mul_add(delta_time, 6.).mul_add(delta_time, 3.));
            });
    }

    #[test]
    fn update_absolute_position() {
        let mut app: TestApp = App::new().with_entity::<PhysicsModule>(()).into();
        let entity1_id =
            app.create_entity::<TestEntity<_>>(Box::new(|b| b.with(Position::xyz(1., 2., 3.))));
        let entity2_id = app.create_child::<TestEntity<_>>(entity1_id, Box::new(|b| b));
        let entity3_id = app.create_child::<TestEntity<_>>(
            entity2_id,
            Box::new(|b| {
                b.with(Position::xyz(4., 5., 6.))
                    .with(Scale::xyz(0.1, 0.2, 0.5))
            }),
        );
        let entity4_id = app.create_child::<TestEntity<_>>(
            entity3_id,
            Box::new(|b| b.with(Position::xyz(7., 8., 9.))),
        );
        app.update();
        app.assert_entity(entity1_id)
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().x, 1.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().y, 2.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().z, 3.));
        app.assert_entity(entity3_id)
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().x, 5.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().y, 7.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().z, 9.));
        app.assert_entity(entity4_id)
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().x, 5.7))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().y, 8.6))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().z, 13.5));
    }

    #[test]
    fn update_absolute_scale() {
        let mut app: TestApp = App::new().with_entity::<PhysicsModule>(()).into();
        let entity1_id = app.create_entity::<TestEntity<_>>(Box::new(|b| {
            b.with(Position::xyz(10., 20., 30.))
                .with(Scale::xyz(1., 2., 3.))
        }));
        let entity2_id = app.create_child::<TestEntity<_>>(entity1_id, Box::new(|b| b));
        let entity3_id = app.create_child::<TestEntity<_>>(
            entity2_id,
            Box::new(|b| {
                b.with(Position::xyz(40., 50., 60.))
                    .with(Scale::xyz(0.1, 0.2, 0.5))
            }),
        );
        let entity4_id = app.create_child::<TestEntity<_>>(
            entity3_id,
            Box::new(|b| b.with(Scale::xyz(4., 3., 2.))),
        );
        let entity5_id = app.create_child::<TestEntity<_>>(
            entity4_id,
            Box::new(|b| {
                b.with(Position::xyz(70., 80., 90.))
                    .with(Scale::xyz(0.5, 0.2, 0.1))
            }),
        );
        app.update();
        app.assert_entity(entity1_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 1.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 2.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 3.));
        app.assert_entity(entity3_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 0.1))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 0.4))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 1.5));
        app.assert_entity(entity4_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 4.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 3.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 2.));
        app.assert_entity(entity5_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 0.05))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 0.08))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 0.15));
    }
}
