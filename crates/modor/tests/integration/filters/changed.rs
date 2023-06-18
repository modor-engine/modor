use modor::{App, BuiltEntity, Changed, Entity, EntityBuilder, Filter, Query, With, World};

#[derive(Component, NoSystem)]
struct TrackedComponent;

#[derive(Component, NoSystem)]
struct Counter(u32);

#[derive(Component)]
struct BaseEntity;

#[systems]
impl BaseEntity {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(TrackedComponent)
            .with(Counter(0))
    }

    #[run_after(component(StaticEntity), component(MutatedEntity))]
    fn update(counter: &mut Counter, _: Filter<Changed<TrackedComponent>>) {
        counter.0 += 1;
    }
}

#[derive(Component, NoSystem)]
struct StaticEntity;

#[derive(Component)]
struct MutatedEntity;

#[systems]
impl MutatedEntity {
    #[run]
    fn update(_position: &mut TrackedComponent) {}
}

#[derive(Component)]
struct UnusedQueryMutatedEntity;

#[systems]
impl UnusedQueryMutatedEntity {
    #[run]
    fn update(_query: Query<'_, (&mut TrackedComponent, Filter<With<Self>>)>) {}
}

#[derive(Component)]
struct ConstQueryMutatedEntity;

#[systems]
impl ConstQueryMutatedEntity {
    #[run]
    fn update(query: Query<'_, (&mut TrackedComponent, Filter<With<Self>>)>) {
        for _ in query.iter() {}
    }
}

#[derive(Component)]
struct MutQueryMutatedEntity;

#[systems]
impl MutQueryMutatedEntity {
    #[run]
    fn update(mut query: Query<'_, (&mut TrackedComponent, Filter<With<Self>>)>) {
        for _ in query.iter_mut() {}
    }
}

#[derive(Component)]
struct OverwrittenEntity;

#[systems]
impl OverwrittenEntity {
    #[run]
    fn update(entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), TrackedComponent);
    }
}

#[modor_test(disabled(wasm))]
fn filter_by_changed_component() {
    App::new()
        .with_entity(BaseEntity::build().with(StaticEntity))
        .with_entity(BaseEntity::build().with(MutatedEntity))
        .with_entity(BaseEntity::build().with(UnusedQueryMutatedEntity))
        .with_entity(BaseEntity::build().with(ConstQueryMutatedEntity))
        .with_entity(BaseEntity::build().with(MutQueryMutatedEntity))
        .with_entity(BaseEntity::build().with(OverwrittenEntity))
        .updated()
        .updated()
        .assert::<With<StaticEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 1)))
        .assert::<With<MutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 2)))
        .assert::<With<UnusedQueryMutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 1)))
        .assert::<With<ConstQueryMutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 1)))
        .assert::<With<MutQueryMutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 2)))
        .assert::<With<OverwrittenEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 2)))
        .with_entity(BaseEntity::build().with(StaticEntity))
        .updated()
        .updated()
        .assert_any::<With<StaticEntity>>(2, |e| {
            e.has(|c: &Counter| assert_eq!(c.0, 1))
                .has(|c: &Counter| assert_eq!(c.0, 2))
        });
}
