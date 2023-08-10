use crate::system_params::{
    assert_iter, Enabled, QueryTester, Value, DISABLED_ID, MATCHING1_ID, MATCHING2_ID, MISSING_ID,
    NO_VALUE_ID, VALUE1, VALUE2,
};
use modor::{Custom, Filter, With};

// TODO: use Custom type when useful

#[modor_test]
fn run_query_iter() {
    QueryTester::<Custom<NamedSystemParam<'_>>>::run(|q| {
        assert_iter(q.iter().map(|v| v.value.0), [VALUE1, VALUE2]);
        assert_iter(q.iter().rev().map(|v| v.value.0), [VALUE2, VALUE1]);
    });
    QueryTester::<Custom<UnnamedSystemParam<'_>>>::run(|q| {
        assert_iter(q.iter().map(|v| v.0 .0), [VALUE1, VALUE2]);
        assert_iter(q.iter().rev().map(|v| v.0 .0), [VALUE2, VALUE1]);
    });
    QueryTester::<Custom<UnitSystemParam>>::run(|q| {
        assert_eq!(q.iter().count(), 6);
        assert_eq!(q.iter().rev().count(), 6);
    });
}

#[modor_test]
fn run_query_iter_mut() {
    QueryTester::<Custom<NamedSystemParam<'_>>>::run(|q| {
        assert_iter(q.iter_mut().map(|v| v.value.0), [VALUE1, VALUE2]);
        assert_iter(q.iter_mut().rev().map(|v| v.value.0), [VALUE2, VALUE1]);
    });
    QueryTester::<Custom<UnnamedSystemParam<'_>>>::run(|q| {
        assert_iter(q.iter_mut().map(|v| v.0 .0), [VALUE1, VALUE2]);
        assert_iter(q.iter_mut().rev().map(|v| v.0 .0), [VALUE2, VALUE1]);
    });
    QueryTester::<Custom<UnitSystemParam>>::run(|q| {
        assert_eq!(q.iter_mut().count(), 6);
        assert_eq!(q.iter_mut().rev().count(), 6);
    });
}

#[modor_test]
fn run_query_get() {
    QueryTester::<Custom<NamedSystemParam<'_>>>::run(|q| {
        assert_eq!(q.get(MISSING_ID).map(|v| v.value.0), None);
        assert_eq!(q.get(DISABLED_ID).map(|v| v.value.0), None);
        assert_eq!(q.get(NO_VALUE_ID).map(|v| v.value.0), None);
        assert_eq!(q.get(MATCHING1_ID).map(|v| v.value.0), Some(VALUE1));
        assert_eq!(q.get(MATCHING2_ID).map(|v| v.value.0), Some(VALUE2));
    });
    QueryTester::<Custom<UnnamedSystemParam<'_>>>::run(|q| {
        assert_eq!(q.get(MISSING_ID).map(|v| v.0 .0), None);
        assert_eq!(q.get(DISABLED_ID).map(|v| v.0 .0), None);
        assert_eq!(q.get(NO_VALUE_ID).map(|v| v.0 .0), None);
        assert_eq!(q.get(MATCHING1_ID).map(|v| v.0 .0), Some(VALUE1));
        assert_eq!(q.get(MATCHING2_ID).map(|v| v.0 .0), Some(VALUE2));
    });
}

#[modor_test]
fn run_query_get_mut() {
    QueryTester::<Custom<NamedSystemParam<'_>>>::run(|q| {
        assert_eq!(q.get_mut(MISSING_ID).map(|v| v.value.0), None);
        assert_eq!(q.get_mut(DISABLED_ID).map(|v| v.value.0), None);
        assert_eq!(q.get_mut(NO_VALUE_ID).map(|v| v.value.0), None);
        assert_eq!(q.get_mut(MATCHING1_ID).map(|v| v.value.0), Some(VALUE1));
        assert_eq!(q.get_mut(MATCHING2_ID).map(|v| v.value.0), Some(VALUE2));
    });
    QueryTester::<Custom<UnnamedSystemParam<'_>>>::run(|q| {
        assert_eq!(q.get_mut(MISSING_ID).map(|v| v.0 .0), None);
        assert_eq!(q.get_mut(DISABLED_ID).map(|v| v.0 .0), None);
        assert_eq!(q.get_mut(NO_VALUE_ID).map(|v| v.0 .0), None);
        assert_eq!(q.get_mut(MATCHING1_ID).map(|v| v.0 .0), Some(VALUE1));
        assert_eq!(q.get_mut(MATCHING2_ID).map(|v| v.0 .0), Some(VALUE2));
    });
}

#[modor_test]
fn run_query_get_both_mut() {
    QueryTester::<Custom<NamedSystemParam<'_>>>::run(|q| {
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING2_ID);
        assert_eq!(left.map(|v| v.value.0), Some(VALUE1));
        assert_eq!(right.map(|v| v.value.0), Some(VALUE2));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.value.0), Some(VALUE2));
        assert_eq!(right.map(|v| v.value.0), Some(VALUE1));
        let (left, right) = q.get_both_mut(MATCHING1_ID, MISSING_ID);
        assert_eq!(left.map(|v| v.value.0), Some(VALUE1));
        assert_eq!(right.map(|v| v.value.0), None);
        let (left, right) = q.get_both_mut(MISSING_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.value.0), None);
        assert_eq!(right.map(|v| v.value.0), Some(VALUE1));
        let (left, right) = q.get_both_mut(MISSING_ID, DISABLED_ID);
        assert_eq!(left.map(|v| v.value.0), None);
        assert_eq!(right.map(|v| v.value.0), None);
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.value.0), Some(VALUE1));
        assert_eq!(right.map(|v| v.value.0), None);
    });
    QueryTester::<Custom<UnnamedSystemParam<'_>>>::run(|q| {
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING2_ID);
        assert_eq!(left.map(|v| v.0 .0), Some(VALUE1));
        assert_eq!(right.map(|v| v.0 .0), Some(VALUE2));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0 .0), Some(VALUE2));
        assert_eq!(right.map(|v| v.0 .0), Some(VALUE1));
        let (left, right) = q.get_both_mut(MATCHING1_ID, MISSING_ID);
        assert_eq!(left.map(|v| v.0 .0), Some(VALUE1));
        assert_eq!(right.map(|v| v.0 .0), None);
        let (left, right) = q.get_both_mut(MISSING_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), Some(VALUE1));
        let (left, right) = q.get_both_mut(MISSING_ID, DISABLED_ID);
        assert_eq!(left.map(|v| v.0 .0), None);
        assert_eq!(right.map(|v| v.0 .0), None);
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0 .0), Some(VALUE1));
        assert_eq!(right.map(|v| v.0 .0), None);
    });
}

#[derive(QuerySystemParam)]
struct NamedSystemParam<'a> {
    value: &'a mut Value,
    _filter: Filter<With<Enabled>>,
}

#[derive(QuerySystemParam)]
struct UnnamedSystemParam<'a>(&'a mut Value, Filter<With<Enabled>>);

#[derive(QuerySystemParam)]
struct UnitSystemParam;
