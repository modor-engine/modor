use fxhash::FxHashSet;
use modor::{App, BuiltEntity, EntityBuilder, Filter, Or, Query, With, Without};
use std::fmt::Debug;
use std::hash::Hash;

fn assert_unordered_iter<T>(actual: impl Iterator<Item = T>, expected: impl IntoIterator<Item = T>)
where
    T: Eq + Hash + Debug,
{
    let actual: FxHashSet<T> = actual.collect();
    let expected: FxHashSet<T> = expected.into_iter().collect();
    assert_eq!(actual, expected);
}

#[derive(Component, NoSystem, PartialEq, Eq, Hash, Debug)]
struct C1(u32);

#[derive(Component, NoSystem, PartialEq, Eq, Hash, Debug)]
struct C2;

#[derive(Component, NoSystem, PartialEq, Eq, Hash, Debug)]
struct C3;

#[derive(Component, NoSystem, PartialEq, Eq, Hash, Debug)]
struct C4;

#[derive(SingletonComponent)]
struct ResultCollector {
    done: bool,
}

#[systems]
impl ResultCollector {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new().with(Self { done: false })
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    #[run]
    fn run_filtered_queries(
        &mut self,
        with: Query<'_, (&C1, Filter<With<C2>>)>,
        without: Query<'_, (&C1, Filter<Without<C2>>)>,
        (and_empty, and_1_with, and_many_with, and_1_without, and_many_without): (
            Query<'_, (&C1, Filter<()>)>,
            Query<'_, (&C1, Filter<(With<C2>,)>)>,
            Query<'_, (&C1, Filter<(With<C2>, With<C3>)>)>,
            Query<'_, (&C1, Filter<(Without<C2>,)>)>,
            Query<'_, (&C1, Filter<(Without<C2>, Without<C3>)>)>,
        ),
        (or_empty, or_1_with, or_many_with, or_1_without, or_many_without): (
            Query<'_, (&C1, Filter<Or<()>>)>,
            Query<'_, (&C1, Filter<Or<(With<C2>,)>>)>,
            Query<'_, (&C1, Filter<Or<(With<C2>, With<C3>)>>)>,
            Query<'_, (&C1, Filter<Or<(Without<C2>,)>>)>,
            Query<'_, (&C1, Filter<Or<(Without<C2>, Without<C3>)>>)>,
        ),
        nested: Query<'_, (&C1, Filter<Or<(With<C3>, (With<C1>, With<C2>))>>)>,
    ) {
        assert_unordered_iter(with.iter().map(|(v, _)| v.0), [1, 3]);
        assert_unordered_iter(without.iter().map(|(v, _)| v.0), [2, 4]);
        assert_unordered_iter(and_empty.iter().map(|(v, _)| v.0), [1, 2, 3, 4]);
        assert_unordered_iter(and_1_with.iter().map(|(v, _)| v.0), [1, 3]);
        assert_unordered_iter(and_many_with.iter().map(|(v, _)| v.0), [1]);
        assert_unordered_iter(and_1_without.iter().map(|(v, _)| v.0), [2, 4]);
        assert_unordered_iter(and_many_without.iter().map(|(v, _)| v.0), [4]);
        assert_unordered_iter(or_empty.iter().map(|(v, _)| v.0), []);
        assert_unordered_iter(or_1_with.iter().map(|(v, _)| v.0), [1, 3]);
        assert_unordered_iter(or_many_with.iter().map(|(v, _)| v.0), [1, 2, 3]);
        assert_unordered_iter(or_1_without.iter().map(|(v, _)| v.0), [2, 4]);
        assert_unordered_iter(or_many_without.iter().map(|(v, _)| v.0), [2, 3, 4]);
        assert_unordered_iter(nested.iter().map(|(v, _)| v.0), [1, 2, 3]);
        self.done = true;
    }
}

#[test]
fn filter_entities_in_query() {
    App::new()
        .with_entity(ResultCollector::build())
        .with_entity(EntityBuilder::new().with(C1(1)).with(C2).with(C3).with(C4))
        .with_entity(EntityBuilder::new().with(C1(2)).with(C3))
        .with_entity(EntityBuilder::new().with(C1(3)).with(C2))
        .with_entity(EntityBuilder::new().with(C1(4)))
        .updated()
        .assert::<With<ResultCollector>>(1, |e| e.has(|c: &ResultCollector| assert!(c.done)));
}
