use crate::system_params::{
    assert_iter, entities, DisabledFilter, Enabled, Matching1Filter, Matching2Filter,
    NoValueFilter, QueryTester, DISABLED_ID, MATCHING1_ID, MATCHING2_CLONE_ID, MATCHING2_ID,
    MISSING_ID, NO_VALUE_ID,
};
use modor::{App, Filter, With};

#[modor_test]
fn run_query_iter() {
    QueryTester::<()>::run(|q| {
        assert_iter(q.iter(), [(), (), (), (), (), (), ()]);
        assert_iter(q.iter().rev(), [(), (), (), (), (), (), ()]);
    });
}

#[modor_test]
fn run_query_iter_mut() {
    QueryTester::<()>::run(|q| {
        assert_iter(q.iter_mut(), [(), (), (), (), (), (), ()]);
        assert_iter(q.iter_mut().rev(), [(), (), (), (), (), (), ()]);
    });
}

#[modor_test]
fn run_query_get() {
    QueryTester::<()>::run(|q| {
        assert_eq!(q.get(MISSING_ID), None);
        assert_eq!(q.get(DISABLED_ID), Some(()));
        assert_eq!(q.get(NO_VALUE_ID), Some(()));
        assert_eq!(q.get(MATCHING1_ID), Some(()));
        assert_eq!(q.get(MATCHING2_ID), Some(()));
    });
}

#[modor_test]
fn run_query_get_mut() {
    QueryTester::<()>::run(|q| {
        assert_eq!(q.get_mut(MISSING_ID), None);
        assert_eq!(q.get_mut(DISABLED_ID), Some(()));
        assert_eq!(q.get_mut(NO_VALUE_ID), Some(()));
        assert_eq!(q.get_mut(MATCHING1_ID), Some(()));
        assert_eq!(q.get_mut(MATCHING2_ID), Some(()));
    });
}

#[modor_test]
fn run_query_get_both_mut() {
    QueryTester::<()>::run(|q| {
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING2_ID);
        assert_eq!(left, Some(()));
        assert_eq!(right, Some(()));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING1_ID);
        assert_eq!(left, Some(()));
        assert_eq!(right, Some(()));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING2_CLONE_ID);
        assert_eq!(left, Some(()));
        assert_eq!(right, Some(()));
        let (left, right) = q.get_both_mut(MATCHING1_ID, MISSING_ID);
        assert_eq!(left, Some(()));
        assert_eq!(right, None);
        let (left, right) = q.get_both_mut(MISSING_ID, MATCHING1_ID);
        assert_eq!(left, None);
        assert_eq!(right, Some(()));
        let (left, right) = q.get_both_mut(MISSING_ID, DISABLED_ID);
        assert_eq!(left, None);
        assert_eq!(right, Some(()));
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING1_ID);
        assert_eq!(left, Some(()));
        assert_eq!(right, None);
    });
}

#[modor_test]
fn run_system_with_param() {
    App::new()
        .with_entity(entities())
        .with_component::<(), _>(Tracked::default)
        .updated()
        .assert::<Matching1Filter>(1, |e| e.has(|t: &Tracked| assert!(t.0)))
        .assert_any::<Matching2Filter>(2, |e| e.has(|t: &Tracked| assert!(t.0)))
        .assert::<DisabledFilter>(1, |e| e.has(|t: &Tracked| assert!(!t.0)))
        .assert::<NoValueFilter>(1, |e| e.has(|t: &Tracked| assert!(t.0)));
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    modor_internal::retry!(60, assert!(are_systems_run_in_parallel!((), ())));
}

#[derive(Component, Default)]
struct Tracked(bool);

#[systems]
impl Tracked {
    #[run]
    fn update(&mut self, _: (), _: Filter<With<Enabled>>) {
        self.0 = true;
    }
}
