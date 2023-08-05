use crate::system_params::{Enabled, Value};
use modor::{Component, Query};

// TODO: remove

#[derive(SystemParam)]
struct SimpleSystemParam<'a: 'a, T: Component> {
    value: &'a Value,
    other_value: &'a T,
    query: Query<'a, &'static Enabled>,
}

#[derive(SystemParam)]
struct TupleSystemParam<'a: 'a, T: Component>(&'a Value, &'a T, Query<'a, &'static Enabled>);

#[test]
fn run() {}
