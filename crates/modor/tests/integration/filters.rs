use fxhash::FxHashSet;
use modor::{App, Built, EntityBuilder, Filter, Or, Query, With, Without};
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

struct ResultCollector {
    done: bool,
}

#[singleton]
impl ResultCollector {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { done: false })
    }

    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    #[run]
    fn run_filtered_queries(
        &mut self,
        with: Query<'_, (&u32, Filter<With<i64>>)>,
        without: Query<'_, (&u32, Filter<Without<i64>>)>,
        (and_empty, and_1_with, and_many_with, and_1_without, and_many_without): (
            Query<'_, (&u32, Filter<()>)>,
            Query<'_, (&u32, Filter<(With<i64>,)>)>,
            Query<'_, (&u32, Filter<(With<i64>, With<u8>)>)>,
            Query<'_, (&u32, Filter<(Without<i64>,)>)>,
            Query<'_, (&u32, Filter<(Without<i64>, Without<u8>)>)>,
        ),
        (or_empty, or_1_with, or_many_with, or_1_without, or_many_without): (
            Query<'_, (&u32, Filter<Or<()>>)>,
            Query<'_, (&u32, Filter<Or<(With<i64>,)>>)>,
            Query<'_, (&u32, Filter<Or<(With<i64>, With<u8>)>>)>,
            Query<'_, (&u32, Filter<Or<(Without<i64>,)>>)>,
            Query<'_, (&u32, Filter<Or<(Without<i64>, Without<u8>)>>)>,
        ),
        nested: Query<'_, (&u32, Filter<Or<(With<u8>, (With<u32>, With<i64>))>>)>,
    ) {
        assert_unordered_iter(with.iter().map(|(&v, _)| v), [1, 3]);
        assert_unordered_iter(without.iter().map(|(&v, _)| v), [2, 4]);
        assert_unordered_iter(and_empty.iter().map(|(&v, _)| v), [1, 2, 3, 4]);
        assert_unordered_iter(and_1_with.iter().map(|(&v, _)| v), [1, 3]);
        assert_unordered_iter(and_many_with.iter().map(|(&v, _)| v), [1]);
        assert_unordered_iter(and_1_without.iter().map(|(&v, _)| v), [2, 4]);
        assert_unordered_iter(and_many_without.iter().map(|(&v, _)| v), [4]);
        assert_unordered_iter(or_empty.iter().map(|(&v, _)| v), []);
        assert_unordered_iter(or_1_with.iter().map(|(&v, _)| v), [1, 3]);
        assert_unordered_iter(or_many_with.iter().map(|(&v, _)| v), [1, 2, 3]);
        assert_unordered_iter(or_1_without.iter().map(|(&v, _)| v), [2, 4]);
        assert_unordered_iter(or_many_without.iter().map(|(&v, _)| v), [2, 3, 4]);
        assert_unordered_iter(nested.iter().map(|(&v, _)| v), [1, 2, 3]);
        self.done = true;
    }
}

struct TestEntity;

#[entity]
impl TestEntity {}

#[test]
fn filter_entities_in_query() {
    App::new()
        .with_entity(ResultCollector::build())
        .with_entity(
            EntityBuilder::new(TestEntity)
                .with(1_u32)
                .with(10_i64)
                .with(100_u8)
                .with(1000_i16),
        )
        .with_entity(EntityBuilder::new(TestEntity).with(2_u32).with(200_u8))
        .with_entity(EntityBuilder::new(TestEntity).with(3_u32).with(30_i64))
        .with_entity(EntityBuilder::new(TestEntity).with(4_u32))
        .updated()
        .assert::<With<ResultCollector>>(1, |e| e.has(|c: &ResultCollector| assert!(c.done)));
}
