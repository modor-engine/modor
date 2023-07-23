#![allow(clippy::redundant_closure_for_method_calls, clippy::option_option)]

use modor::{App, BuiltEntity, EntityBuilder, Not, Query, QuerySystemParam, With};
use std::fmt::Debug;

fn assert_iter<T, E, I1, I2>(mut actual: I1, expected: E)
where
    T: PartialEq + Debug,
    I1: Iterator<Item = T> + ExactSizeIterator,
    I2: ExactSizeIterator + Iterator<Item = T>,
    E: IntoIterator<Item = T, IntoIter = I2>,
{
    let expected_iter = expected.into_iter();
    let expected_len = expected_iter.len();
    for (pos, expected_item) in expected_iter.enumerate() {
        assert_eq!(
            actual.len(),
            expected_len - pos,
            "wrong size at position {pos}"
        );
        assert_eq!(
            actual.next(),
            Some(expected_item),
            "wrong item at position {pos}"
        );
    }
    assert_eq!(actual.len(), 0, "size not zero after last item");
    assert_eq!(actual.next(), None, "more items than expected");
}

const ROOT_ID: usize = 1;
const MATCHING1_ID: usize = 2;
const DISABLED_ID: usize = 3;
const NO_VALUE_ID: usize = 4;
const MATCHING2_ID: usize = 5;
const MISSING_ID: usize = 100;
const VALUE1: u32 = 1;
const VALUE2: u32 = 3;
const OTHER_VALUE2: u32 = 4;

type Matching1Filter = (With<Value>, Not<With<OtherValue>>, With<Enabled>);
type Matching2Filter = (With<Value>, With<OtherValue>, With<Enabled>);
type NoValueFilter = (Not<With<Value>>, With<Enabled>);
type DisabledFilter = (With<Value>, Not<With<Enabled>>);

fn entities() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(entity(Some(VALUE1), None, true).child_entity(entity(Some(2), None, false)))
        .child_entity(entity(None, None, true).child_entity(entity(
            Some(VALUE2),
            Some(OTHER_VALUE2),
            true,
        )))
}

fn entity(value: Option<u32>, other_value: Option<u32>, is_enabled: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component_option(value.map(Value))
        .component_option(other_value.map(OtherValue))
        .component_option(is_enabled.then_some(Enabled))
}

#[derive(SingletonComponent, NoSystem)]
struct ValueSingleton(u32);

#[derive(Component, NoSystem)]
pub struct Value(u32);

#[derive(Component, NoSystem)]
pub struct OtherValue(u32);

#[derive(Component, NoSystem)]
pub struct Enabled;

#[derive(Component)]
struct QueryTester<P>
where
    P: 'static + QuerySystemParam,
{
    test_fn: fn(&mut Query<'_, P>),
    is_done: bool,
}

#[systems]
impl<P> QueryTester<P>
where
    P: 'static + QuerySystemParam,
{
    fn run(test_fn: fn(&mut Query<'_, P>)) -> App {
        App::new()
            .with_entity(Self {
                test_fn,
                is_done: false,
            })
            .with_entity(entities())
            .updated()
            .assert::<With<Self>>(1, |e| e.has(|t: &Self| assert!(t.is_done)))
    }

    #[run]
    fn update(&mut self, mut query: Query<'_, P>) {
        (self.test_fn)(&mut query);
        self.is_done = true;
    }
}

macro_rules! are_systems_run_in_parallel {
    ($query_system_param:ty, $system_param:ty) => {{
        #[derive(Component)]
        struct ParallelTester;

        #[systems]
        impl ParallelTester {
            #[run]
            fn system1(_: modor::Query<'_, $query_system_param>, _: $system_param) {
                spin_sleep::sleep(std::time::Duration::from_millis(100));
            }

            #[run]
            fn system2(_: modor::Query<'_, $query_system_param>, _: $system_param) {
                spin_sleep::sleep(std::time::Duration::from_millis(100));
            }
        }

        let now = instant::Instant::now();
        #[allow(unused_qualifications)]
        modor::App::new()
            .with_thread_count(2)
            .with_entity(ParallelTester)
            .with_entity(crate::system_params::ValueSingleton(10))
            .updated();
        now.elapsed() < std::time::Duration::from_millis(200)
    }};
}

pub mod components;
pub mod components_mut;
pub mod entity;
pub mod entity_mut;
pub mod optional_components;
pub mod optional_components_mut;
pub mod optional_singleton;
pub mod optional_singleton_mut;
pub mod singleton;
pub mod singleton_mut;
pub mod tuples_empty;
pub mod tuples_many_items;
pub mod tuples_one_item;
pub mod world;
