use log::Level;
use modor::{App, Glob, Globals};
use modor_derive::{Node, RootNode, Visit};

#[modor::test]
fn create_glob() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let glob2 = Glob::new(&mut ctx, "b");
    assert_eq!(glob1.index(), 0);
    assert_eq!(glob2.index(), 1);
    assert_eq!(glob1.get(&mut ctx), &"a");
    assert_eq!(glob2.get(&mut ctx), &"b");
    *glob1.get_mut(&mut ctx) = "c";
    assert_eq!(glob1.get(&mut ctx), &"c");
    assert_eq!(glob2.get(&mut ctx), &"b");
}

#[modor::test]
fn create_glob_ref() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let glob2 = Glob::new(&mut ctx, "b");
    let glob1_ref = glob1.as_ref().clone();
    let glob2_ref = glob2.as_ref().clone();
    assert_eq!(glob1_ref.index(), 0);
    assert_eq!(glob2_ref.index(), 1);
    assert_eq!(glob1_ref.get(&mut ctx), &"a");
    assert_eq!(glob2_ref.get(&mut ctx), &"b");
    *glob1.get_mut(&mut ctx) = "c";
    assert_eq!(glob1_ref.get(&mut ctx), &"c");
    assert_eq!(glob2_ref.get(&mut ctx), &"b");
}

#[modor::test]
fn recreate_glob_without_ref_before_update() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let _glob2 = Glob::new(&mut ctx, "b");
    drop(glob1);
    let glob3 = Glob::new(&mut ctx, "c");
    assert_eq!(glob3.index(), 2);
    let glob4 = Glob::new(&mut ctx, "d");
    assert_eq!(glob4.index(), 3);
}

#[modor::test]
fn recreate_glob_without_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let _glob2 = Glob::new(&mut ctx, "b");
    drop(glob1);
    app.update();
    let mut ctx = app.ctx();
    let glob3 = Glob::new(&mut ctx, "c");
    assert_eq!(glob3.index(), 0);
    let glob4 = Glob::new(&mut ctx, "d");
    assert_eq!(glob4.index(), 2);
}

#[modor::test]
fn recreate_glob_with_not_dropped_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let _glob2 = Glob::new(&mut ctx, "b");
    let _glob1_ref = glob1.as_ref().clone();
    drop(glob1);
    app.update();
    let mut ctx = app.ctx();
    let glob3 = Glob::new(&mut ctx, "c");
    assert_eq!(glob3.index(), 2);
}

#[modor::test]
fn recreate_glob_with_dropped_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let _glob2 = Glob::new(&mut ctx, "b");
    let glob1_ref = glob1.as_ref().clone();
    drop(glob1);
    drop(glob1_ref);
    app.update();
    let mut ctx = app.ctx();
    let glob3 = Glob::new(&mut ctx, "c");
    assert_eq!(glob3.index(), 0);
}

#[modor::test]
fn access_all_globals() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let _glob1 = Glob::new(&mut ctx, "a");
    let _glob2 = Glob::new(&mut ctx, "b");
    let globals = app.root::<Globals<&str>>();
    assert!(globals.deleted_items().is_empty());
    assert_eq!(globals.get(0), Some(&"a"));
    assert_eq!(globals.get(1), Some(&"b"));
    assert_eq!(globals.get(2), None);
    assert_eq!(globals.iter().collect::<Vec<_>>(), vec![&"a", &"b"]);
    assert_eq!(globals.into_iter().collect::<Vec<_>>(), vec![&"a", &"b"]);
    let enumerated_values = globals.iter_enumerated().collect::<Vec<_>>();
    assert_eq!(enumerated_values, vec![(0, &"a"), (1, &"b")]);
}

#[modor::test]
fn access_all_globals_after_value_dropped() {
    let mut app = App::new::<Root>(Level::Info);
    let mut ctx = app.ctx();
    let glob1 = Glob::new(&mut ctx, "a");
    let _glob2 = Glob::new(&mut ctx, "b");
    drop(glob1);
    app.update();
    let globals = app.root::<Globals<&str>>();
    assert_eq!(globals.deleted_items(), [(0, "a")]);
    assert_eq!(globals.get(0), None);
    assert_eq!(globals.get(1), Some(&"b"));
    assert_eq!(globals.get(2), None);
    assert_eq!(globals.iter().collect::<Vec<_>>(), vec![&"b"]);
    assert_eq!(globals.into_iter().collect::<Vec<_>>(), vec![&"b"]);
    let enumerated_values = globals.iter_enumerated().collect::<Vec<_>>();
    assert_eq!(enumerated_values, vec![(1, &"b")]);
}

#[modor::test]
fn access_glob() {}

#[derive(Default, RootNode, Node, Visit)]
struct Root;
