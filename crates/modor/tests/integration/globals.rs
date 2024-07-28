use log::Level;
use modor::{App, FromApp, Glob, Global, Globals};
use modor_derive::State;

#[modor::test]
fn create_glob() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let glob2 = Glob::<Label>::from_app(&mut app);
    glob1.get_mut(&mut app).0 += "a";
    glob2.get_mut(&mut app).0 += "b";
    assert_eq!(glob1.index(), 0);
    assert_eq!(glob2.index(), 1);
    assert_eq!(glob1.get(&app).0, "0a");
    assert_eq!(glob2.get(&app).0, "1b");
    glob1.get_mut(&mut app).0 += "c";
    assert_eq!(glob1.get(&app).0, "0ac");
    assert_eq!(glob2.get(&app).0, "1b");
}

#[modor::test]
fn create_glob_ref() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let glob2 = Glob::<Label>::from_app(&mut app);
    glob1.get_mut(&mut app).0 += "a";
    glob2.get_mut(&mut app).0 += "b";
    let glob1_ref = glob1.to_ref();
    let glob2_ref = glob2.to_ref();
    assert_eq!(glob1_ref.index(), 0);
    assert_eq!(glob2_ref.index(), 1);
    assert_eq!(glob1_ref.get(&app).0, "0a");
    assert_eq!(glob2_ref.get(&app).0, "1b");
    glob1.get_mut(&mut app).0 += "c";
    assert_eq!(glob1_ref.get(&app).0, "0ac");
    assert_eq!(glob2_ref.get(&app).0, "1b");
}

#[modor::test]
fn recreate_glob_without_ref_before_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let _glob2 = Glob::<Label>::from_app(&mut app);
    drop(glob1);
    let glob3 = Glob::<Label>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
    let glob4 = Glob::<Label>::from_app(&mut app);
    assert_eq!(glob4.index(), 3);
}

#[modor::test]
fn recreate_glob_without_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let _glob2 = Glob::<Label>::from_app(&mut app);
    drop(glob1);
    app.update();
    let glob3 = Glob::<Label>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
    app.update();
    assert_eq!(Glob::<Label>::from_app(&mut app).index(), 0);
    assert_eq!(Glob::<Label>::from_app(&mut app).index(), 3);
}

#[modor::test]
fn recreate_glob_with_not_dropped_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let _glob2 = Glob::<Label>::from_app(&mut app);
    let _glob1_ref = glob1.to_ref();
    drop(glob1);
    app.update();
    let glob3 = Glob::<Label>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
}

#[modor::test]
fn recreate_glob_with_dropped_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let _glob2 = Glob::<Label>::from_app(&mut app);
    let glob1_ref = glob1.to_ref();
    drop(glob1);
    drop(glob1_ref);
    app.update();
    let glob3 = Glob::<Label>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
    app.update();
    assert_eq!(Glob::<Label>::from_app(&mut app).index(), 0);
    assert_eq!(Glob::<Label>::from_app(&mut app).index(), 3);
}

#[modor::test]
fn access_all_globals() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let glob2 = Glob::<Label>::from_app(&mut app);
    glob1.get_mut(&mut app).0 += "a";
    glob2.get_mut(&mut app).0 += "b";
    let globals = app.get_mut::<Globals<Label>>();
    assert!(globals.deleted_items().is_empty());
    assert_eq!(globals.get(0).map(|l| l.0.as_str()), Some("0a"));
    assert_eq!(globals.get(1).map(|l| l.0.as_str()), Some("1b"));
    assert_eq!(globals.get(2).map(|l| l.0.as_str()), None);
    assert_eq!(globals.get_mut(0).map(|l| l.0.as_str()), Some("0a"));
    assert_eq!(globals.get_mut(1).map(|l| l.0.as_str()), Some("1b"));
    assert_eq!(globals.get_mut(2).map(|l| l.0.as_str()), None);
    globals[&glob1].0 += "a";
    assert_eq!(globals[&glob1].0, "0aa");
    assert_eq!(globals[&glob2].0, "1b");
    let iterator: Vec<_> = globals.iter().map(|l| l.0.as_str()).collect();
    assert_eq!(iterator, vec!["0aa", "1b"]);
    let iterator: Vec<_> = globals.iter_mut().map(|l| l.0.as_str()).collect();
    assert_eq!(iterator, vec!["0aa", "1b"]);
    let iterator: Vec<_> = (&*globals).into_iter().map(|l| l.0.as_str()).collect();
    assert_eq!(iterator, vec!["0aa", "1b"]);
    let iterator: Vec<_> = (&mut *globals).into_iter().map(|l| l.0.as_str()).collect();
    assert_eq!(iterator, vec!["0aa", "1b"]);
    let iterator: Vec<_> = globals
        .iter_enumerated()
        .map(|(i, l)| (i, l.0.as_str()))
        .collect();
    assert_eq!(iterator, vec![(0, "0aa"), (1, "1b")]);
    let iterator: Vec<_> = globals
        .iter_mut_enumerated()
        .map(|(i, l)| (i, l.0.as_str()))
        .collect();
    assert_eq!(iterator, vec![(0, "0aa"), (1, "1b")]);
}

#[modor::test]
fn access_all_globals_after_value_dropped() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    let glob2 = Glob::<Label>::from_app(&mut app);
    glob1.get_mut(&mut app).0 += "a";
    glob2.get_mut(&mut app).0 += "b";
    drop(glob1);
    app.update();
    let globals = app.get_mut::<Globals<Label>>();
    let deleted_items = globals.deleted_items();
    assert_eq!(deleted_items.len(), 1);
    assert_eq!(deleted_items[0].0, 0);
    assert_eq!(deleted_items[0].1 .0, "0a");
    assert_eq!(globals.get(0).map(|l| l.0.as_str()), None);
    assert_eq!(globals.get(1).map(|l| l.0.as_str()), Some("1b"));
    assert_eq!(globals.get(2).map(|l| l.0.as_str()), None);
    let iterator: Vec<_> = globals.iter().map(|l| l.0.as_str()).collect();
    assert_eq!(iterator, vec!["1b"]);
    let iterator: Vec<_> = globals
        .iter_enumerated()
        .map(|(i, l)| (i, l.0.as_str()))
        .collect();
    assert_eq!(iterator, vec![(1, "1b")]);
}

#[modor::test]
fn take_glob() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<Label>::from_app(&mut app);
    glob1.get_mut(&mut app).0 += "a";
    let result = glob1.take(&mut app, |glob1, _app| {
        assert_eq!(glob1.0, "0a");
        42
    });
    assert_eq!(result, 42);
}

#[modor::test]
fn access_glob() {}

#[derive(Default, State)]
struct Root;

#[derive(FromApp)]
struct Label(String);

impl Global for Label {
    fn init(&mut self, _app: &mut App, index: usize) {
        self.0 = index.to_string();
    }
}
