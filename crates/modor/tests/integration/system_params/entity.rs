use crate::system_params::{
    assert_iter, entities, DisabledFilter, Enabled, Matching1Filter, Matching2Filter,
    NoValueFilter, QueryTester, Value, DISABLED_ID, MATCHING1_ID, MATCHING2_CLONE_ID, MATCHING2_ID,
    MISSING_ID, NO_VALUE_ID, ROOT_ID,
};
use modor::{App, Entity, Filter, With};

#[modor_test]
fn run_query_iter() {
    QueryTester::<(Entity<'_>, Filter<(With<Value>, With<Enabled>)>)>::run(|q| {
        let ids = [MATCHING1_ID, MATCHING2_ID, MATCHING2_CLONE_ID];
        assert_iter(q.iter().map(|v| v.0.id()), ids);
        let ids = [MATCHING2_CLONE_ID, MATCHING2_ID, MATCHING1_ID];
        assert_iter(q.iter().rev().map(|v| v.0.id()), ids);
    });
}

#[modor_test]
fn run_query_iter_mut() {
    QueryTester::<(Entity<'_>, Filter<(With<Value>, With<Enabled>)>)>::run(|q| {
        let ids = [MATCHING1_ID, MATCHING2_ID, MATCHING2_CLONE_ID];
        assert_iter(q.iter_mut().map(|v| v.0.id()), ids);
        let ids = [MATCHING2_CLONE_ID, MATCHING2_ID, MATCHING1_ID];
        assert_iter(q.iter_mut().rev().map(|v| v.0.id()), ids);
    });
}

#[modor_test]
fn run_query_get() {
    QueryTester::<(Entity<'_>, Filter<(With<Value>, With<Enabled>)>)>::run(|q| {
        assert_eq!(q.get(MISSING_ID).map(|v| v.0.id()), None);
        assert_eq!(q.get(DISABLED_ID).map(|v| v.0.id()), None);
        assert_eq!(q.get(NO_VALUE_ID).map(|v| v.0.id()), None);
        assert_eq!(q.get(MATCHING1_ID).map(|v| v.0.id()), Some(MATCHING1_ID));
        assert_eq!(q.get(MATCHING2_ID).map(|v| v.0.id()), Some(MATCHING2_ID));
    });
}

#[modor_test]
fn run_query_get_mut() {
    QueryTester::<(Entity<'_>, Filter<(With<Value>, With<Enabled>)>)>::run(|q| {
        assert_eq!(q.get_mut(MISSING_ID).map(|v| v.0.id()), None);
        assert_eq!(q.get_mut(DISABLED_ID).map(|v| v.0.id()), None);
        assert_eq!(q.get_mut(NO_VALUE_ID).map(|v| v.0.id()), None);
        let id = Some(MATCHING1_ID);
        assert_eq!(q.get_mut(MATCHING1_ID).map(|v| v.0.id()), id);
        let id = Some(MATCHING2_ID);
        assert_eq!(q.get_mut(MATCHING2_ID).map(|v| v.0.id()), id);
    });
}

#[modor_test]
fn run_query_get_both_mut() {
    QueryTester::<(Entity<'_>, Filter<(With<Value>, With<Enabled>)>)>::run(|q| {
        let (left, right) = q.get_both_mut(MATCHING1_ID, MATCHING2_ID);
        assert_eq!(left.map(|v| v.0.id()), Some(MATCHING1_ID));
        assert_eq!(right.map(|v| v.0.id()), Some(MATCHING2_ID));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0.id()), Some(MATCHING2_ID));
        assert_eq!(right.map(|v| v.0.id()), Some(MATCHING1_ID));
        let (left, right) = q.get_both_mut(MATCHING2_ID, MATCHING2_CLONE_ID);
        assert_eq!(left.map(|v| v.0.id()), Some(MATCHING2_ID));
        assert_eq!(right.map(|v| v.0.id()), Some(MATCHING2_CLONE_ID));
        let (left, right) = q.get_both_mut(MATCHING1_ID, MISSING_ID);
        assert_eq!(left.map(|v| v.0.id()), Some(MATCHING1_ID));
        assert_eq!(right.map(|v| v.0.id()), None);
        let (left, right) = q.get_both_mut(MISSING_ID, MATCHING1_ID);
        assert_eq!(left.map(|v| v.0.id()), None);
        assert_eq!(right.map(|v| v.0.id()), Some(MATCHING1_ID));
        let (left, right) = q.get_both_mut(MISSING_ID, DISABLED_ID);
        assert_eq!(left.map(|v| v.0.id()), None);
        assert_eq!(right.map(|v| v.0.id()), None);
    });
}

#[modor_test(disabled(wasm))]
fn run_system_with_param() {
    App::new()
        .with_entity(Blank)
        .with_entity(entities())
        .with_component::<(), _>(Tracked::default)
        .updated()
        .assert::<Matching1Filter>(1, |e| {
            e.has(|t: &Tracked| assert_eq!(t.id, Some(MATCHING1_ID)))
                .has(|t: &Tracked| assert_eq!(t.depth, Some(1)))
                .has(|t: &Tracked| assert_eq!(t.parent_id, Some(ROOT_ID)))
                .has(|t: &Tracked| assert_eq!(t.child_ids, vec![DISABLED_ID]))
        })
        .assert_any::<Matching2Filter>(2, |e| {
            e.has(|t: &Tracked| assert_eq!(t.id, Some(MATCHING2_ID)))
                .has(|t: &Tracked| assert_eq!(t.depth, Some(2)))
                .has(|t: &Tracked| assert_eq!(t.parent_id, Some(NO_VALUE_ID)))
                .has(|t: &Tracked| assert!(t.child_ids.is_empty()))
        })
        .assert::<DisabledFilter>(1, |e| e.has(|t: &Tracked| assert_eq!(t.id, None)))
        .assert::<NoValueFilter>(1, |e| e.has(|t: &Tracked| assert_eq!(t.id, None)));
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    modor_internal::retry!(60, assert!(are_systems_run_in_parallel!(Entity<'_>, ())));
}

#[derive(Component, NoSystem)]
struct Blank; // used to align entity IDs with constants

#[derive(Component, Default)]
struct Tracked {
    id: Option<usize>,
    depth: Option<usize>,
    parent_id: Option<usize>,
    child_ids: Vec<usize>,
}

#[systems]
impl Tracked {
    #[run]
    fn update(&mut self, param: Entity<'_>, _: Filter<(With<Value>, With<Enabled>)>) {
        self.id = Some(param.id());
        self.depth = Some(param.depth());
        self.parent_id = param.parent().map(Entity::id);
        self.child_ids = param.children().map(Entity::id).collect();
    }
}
