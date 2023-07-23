use modor::{
    App, BuiltEntity, Changed, EntityAssertions, EntityBuilder, EntityFilter, EntityMut, Filter,
    Not, Or, Query, With,
};

#[modor_test(disabled(wasm))]
fn use_changed_filter() {
    App::new()
        .with_entity(changeable_entity(1).component(UnchangedValue))
        .with_entity(changeable_entity(2).component(MutValue))
        .with_entity(changeable_entity(3).component(UnusedQueryMutValue))
        .with_entity(changeable_entity(4).component(ConstQueryMutValue))
        .with_entity(changeable_entity(5).component(MutQueryMutValue))
        .with_entity(changeable_entity(6).component(OverwrittenValue))
        .updated()
        .updated()
        .assert::<With<UnchangedValue>>(1, |e| e.has(|c: &ChangeCounter| assert_eq!(c.0, 1)))
        .assert::<With<MutValue>>(1, |e| e.has(|c: &ChangeCounter| assert_eq!(c.0, 2)))
        .assert::<With<UnusedQueryMutValue>>(1, |e| e.has(|c: &ChangeCounter| assert_eq!(c.0, 1)))
        .assert::<With<ConstQueryMutValue>>(1, |e| e.has(|c: &ChangeCounter| assert_eq!(c.0, 1)))
        .assert::<With<MutQueryMutValue>>(1, |e| e.has(|c: &ChangeCounter| assert_eq!(c.0, 2)))
        .assert::<With<OverwrittenValue>>(1, |e| e.has(|c: &ChangeCounter| assert_eq!(c.0, 2)))
        .with_entity(changeable_entity(7).component(UnchangedValue))
        .updated()
        .updated()
        .assert_any::<With<UnchangedValue>>(2, |e| {
            e.has(|c: &ChangeCounter| assert_eq!(c.0, 1))
                .has(|c: &ChangeCounter| assert_eq!(c.0, 2))
        });
}

#[modor_test]
fn use_other_filters() {
    App::new()
        .with_entity(entity(0, true, true))
        .with_entity(entity(1, true, false))
        .with_entity(entity(2, false, false))
        .with_entity(entity(3, false, true))
        .assert::<()>(4, assert_values([0, 1, 2, 3]))
        .assert::<With<Filter1>>(2, assert_values([0, 1]))
        .assert::<With<Filter2>>(2, assert_values([0, 3]))
        .assert::<(With<Filter1>, With<Filter2>)>(1, assert_values([0]))
        .assert::<Not<()>>(0, assert_values([]))
        .assert::<Not<With<Filter1>>>(2, assert_values([2, 3]))
        .assert::<Not<With<Filter2>>>(2, assert_values([1, 2]))
        .assert::<Not<(With<Filter1>, With<Filter2>)>>(3, assert_values([1, 2, 3]))
        .assert::<(Not<With<Filter1>>, Not<With<Filter2>>)>(1, assert_values([2]))
        .assert::<Or<()>>(0, assert_values([]))
        .assert::<Or<(With<Filter1>,)>>(2, assert_values([0, 1]))
        .assert::<Or<(With<Filter1>, With<Filter2>)>>(3, assert_values([0, 1, 3]))
        .assert::<Or<(Not<With<Filter1>>,)>>(2, assert_values([2, 3]))
        .assert::<Or<(Not<With<Filter1>>, Not<With<Filter2>>)>>(3, assert_values([1, 2, 3]))
        .assert::<Or<(With<Filter1>, (With<Filter2>, With<Value>))>>(3, assert_values([0, 1, 3]));
}

fn assert_values<F, const N: usize>(
    values: [u32; N],
) -> impl FnOnce(EntityAssertions<'_, F>) -> EntityAssertions<'_, F> + 'static
where
    F: EntityFilter,
{
    move |e| e.has(|v: &Value| assert!(values.contains(&v.0)))
}

fn entity(value: u32, filter1: bool, filter2: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Value(value))
        .component_option(filter1.then_some(Filter1))
        .component_option(filter2.then_some(Filter2))
}

fn changeable_entity(value: u32) -> impl BuiltEntity {
    entity(value, false, false).component(ChangeCounter(0))
}

#[derive(Component, NoSystem)]
struct Value(u32);

#[derive(Component, NoSystem)]
struct Filter1;

#[derive(Component, NoSystem)]
struct Filter2;

#[derive(Component)]
struct ChangeCounter(u32);

#[systems]
impl ChangeCounter {
    #[run_after(
        component(MutValue),
        component(UnusedQueryMutValue),
        component(ConstQueryMutValue),
        component(MutQueryMutValue),
        component(OverwrittenValue)
    )]
    fn update(&mut self, _: Filter<Changed<Value>>) {
        self.0 += 1;
    }
}

#[derive(Component, NoSystem)]
struct UnchangedValue;

#[derive(Component)]
struct MutValue;

#[systems]
impl MutValue {
    #[run]
    fn update(_: &mut Value) {}
}

#[derive(Component)]
struct UnusedQueryMutValue;

#[systems]
impl UnusedQueryMutValue {
    #[run]
    fn update(_: Query<'_, (&mut Value, Filter<With<Self>>)>) {}
}

#[derive(Component)]
struct ConstQueryMutValue;

#[systems]
impl ConstQueryMutValue {
    #[run]
    fn update(query: Query<'_, (&mut Value, Filter<With<Self>>)>) {
        for _ in query.iter() {}
    }
}

#[derive(Component)]
struct MutQueryMutValue;

#[systems]
impl MutQueryMutValue {
    #[run]
    fn update(mut query: Query<'_, (&mut Value, Filter<With<Self>>)>) {
        for _ in query.iter_mut() {}
    }
}

#[derive(Component)]
struct OverwrittenValue;

#[systems]
impl OverwrittenValue {
    #[run]
    fn update(mut entity: EntityMut<'_>) {
        entity.add_component(Value(0));
    }
}
