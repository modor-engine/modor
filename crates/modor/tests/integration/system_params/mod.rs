pub mod components;
pub mod components_mut;
pub mod entity;
pub mod filters;
pub mod optional_components;
pub mod optional_components_mut;
pub mod optional_singletons;
pub mod optional_singletons_mut;
pub mod queries;
pub mod singletons;
pub mod singletons_mut;
pub mod tuples;
pub mod world;
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
            "wrong size at position {}",
            pos
        );
        assert_eq!(
            actual.next(),
            Some(expected_item),
            "wrong item at position {}",
            pos
        );
    }
    assert_eq!(actual.len(), 0, "size not zero after last item");
    assert_eq!(actual.next(), None, "more items than expected");
}

#[derive(Component)]
struct Value(u32);

#[derive(Component, Clone)]
struct OptionalValue(u32);

#[derive(Component)]
struct Text(String);
