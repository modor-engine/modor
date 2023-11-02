use crate::system_params::{
    assert_iter, entities, DisabledFilter, Enabled, Matching1Filter, Matching2Filter,
    NoValueFilter, QueryTester, Value, DISABLED_ID, MATCHING1_ID, MATCHING2_CLONE_ID, MATCHING2_ID,
    MISSING_ID, NO_VALUE_ID, VALUE1, VALUE2, VALUE2_CLONE,
};
use modor::{App, Filter, With};

#[modor_test]
fn run_query_iter() {
    QueryTester::<(Option<&Value>, Filter<With<Enabled>>)>::run(|q| {
        let values = [Some(VALUE1), None, Some(VALUE2), Some(VALUE2_CLONE)];
        assert_iter(q.iter().map(|v| v.0.map(|v| v.0)), values);
        let values = [Some(VALUE2_CLONE), Some(VALUE2), None, Some(VALUE1)];
        assert_iter(q.iter().rev().map(|v| v.0.map(|v| v.0)), values);
    });
}

#[modor_test]
fn run_query_iter_mut() {
    QueryTester::<(Option<&Value>, Filter<With<Enabled>>)>::run(|q| {
        let values = [Some(VALUE1), None, Some(VALUE2), Some(VALUE2_CLONE)];
        assert_iter(q.iter_mut().map(|v| v.0.map(|v| v.0)), values);
        let values = [Some(VALUE2_CLONE), Some(VALUE2), None, Some(VALUE1)];
        assert_iter(q.iter_mut().rev().map(|v| v.0.map(|v| v.0)), values);
    });
}

#[modor_test]
fn run_query_get() {
    QueryTester::<(Option<&Value>, Filter<With<Enabled>>)>::run(|q| {
        assert_eq!(q.get(MISSING_ID).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(q.get(DISABLED_ID).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(q.get(NO_VALUE_ID).map(|v| v.0.map(|v| v.0)), Some(None));
        let value = Some(Some(VALUE1));
        assert_eq!(q.get(MATCHING1_ID).map(|v| v.0.map(|v| v.0)), value);
        let value = Some(Some(VALUE2));
        assert_eq!(q.get(MATCHING2_ID).map(|v| v.0.map(|v| v.0)), value);
    });
}

#[modor_test]
fn run_query_get_mut() {
    QueryTester::<(Option<&Value>, Filter<With<Enabled>>)>::run(|q| {
        assert_eq!(q.get_mut(MISSING_ID).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(q.get_mut(DISABLED_ID).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(q.get_mut(NO_VALUE_ID).map(|v| v.0.map(|v| v.0)), Some(None));
        let value = Some(Some(VALUE1));
        assert_eq!(q.get_mut(MATCHING1_ID).map(|v| v.0.map(|v| v.0)), value);
        let value = Some(Some(VALUE2));
        assert_eq!(q.get_mut(MATCHING2_ID).map(|v| v.0.map(|v| v.0)), value);
    });
}

#[modor_test]
fn run_query_get_both_mut() {
    QueryTester::<(Option<&Value>, Filter<With<Enabled>>)>::run(|q| {
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING2_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE1)));
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE2)));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE2)));
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE1)));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING2_CLONE_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE2)));
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE2_CLONE)));
        let (left, right) = q.get_both_mut(MATCHING1_ID, MISSING_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE1)));
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), None);
        let (left, right) = q.get_both_mut(MISSING_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE1)));
        let (left, right) = q.get_both_mut(MISSING_ID, DISABLED_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), None);
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), Some(Some(VALUE1)));
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), None);
    });
}

#[modor_test]
fn run_system_with_param() {
    App::new()
        .with_entity(entities())
        .with_component::<(), _>(Tracked::default)
        .updated()
        .assert::<Matching1Filter>(1, |e| {
            e.has(|t: &Tracked| assert_eq!(t.0, Some(Some(VALUE1))))
        })
        .assert_any::<Matching2Filter>(2, |e| {
            e.has(|t: &Tracked| assert_eq!(t.0, Some(Some(VALUE2))))
        })
        .assert::<DisabledFilter>(1, |e| e.has(|t: &Tracked| assert_eq!(t.0, None)))
        .assert::<NoValueFilter>(1, |e| e.has(|t: &Tracked| assert_eq!(t.0, Some(None))));
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    modor_internal::retry!(
        60,
        assert!(are_systems_run_in_parallel!(Option<&Value>, ()))
    );
}

#[derive(Component, Default)]
struct Tracked(Option<Option<u32>>);

#[systems]
impl Tracked {
    #[run]
    fn update(&mut self, param: Option<&Value>, _: Filter<With<Enabled>>) {
        self.0 = Some(param.map(|v| v.0));
    }
}
