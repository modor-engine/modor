use log::Level;
use modor::{App, FromApp, Glob, Globals};
use modor_derive::{Node, RootNode};

#[modor::test]
fn create_glob() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::from_app(&mut app);
    let glob2 = Glob::from_app(&mut app);
    *glob1.get_mut(&mut app) = "a";
    *glob2.get_mut(&mut app) = "b";
    assert_eq!(glob1.index(), 0);
    assert_eq!(glob2.index(), 1);
    assert_eq!(glob1.get(&app), &"a");
    assert_eq!(glob2.get(&app), &"b");
    *glob1.get_mut(&mut app) = "c";
    assert_eq!(glob1.get(&app), &"c");
    assert_eq!(glob2.get(&app), &"b");
}

#[modor::test]
fn create_glob_ref() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::from_app(&mut app);
    let glob2 = Glob::from_app(&mut app);
    *glob1.get_mut(&mut app) = "a";
    *glob2.get_mut(&mut app) = "b";
    let glob1_ref = glob1.to_ref();
    let glob2_ref = glob2.to_ref();
    assert_eq!(glob1_ref.index(), 0);
    assert_eq!(glob2_ref.index(), 1);
    assert_eq!(glob1_ref.get(&app), &"a");
    assert_eq!(glob2_ref.get(&app), &"b");
    *glob1.get_mut(&mut app) = "c";
    assert_eq!(glob1_ref.get(&app), &"c");
    assert_eq!(glob2_ref.get(&app), &"b");
}

#[modor::test]
fn recreate_glob_without_ref_before_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::from_app(&mut app);
    let _glob2 = Glob::<&str>::from_app(&mut app);
    *glob1.get_mut(&mut app) = "a";
    drop(glob1);
    let glob3 = Glob::<&str>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
    let glob4 = Glob::<&str>::from_app(&mut app);
    assert_eq!(glob4.index(), 3);
}

#[modor::test]
fn recreate_glob_without_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<&str>::from_app(&mut app);
    let _glob2 = Glob::<&str>::from_app(&mut app);
    drop(glob1);
    app.update();
    let glob3 = Glob::<&str>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
    app.update();
    assert_eq!(Glob::<&str>::from_app(&mut app).index(), 0);
    assert_eq!(Glob::<&str>::from_app(&mut app).index(), 3);
}

#[modor::test]
fn recreate_glob_with_not_dropped_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<&str>::from_app(&mut app);
    let _glob2 = Glob::<&str>::from_app(&mut app);
    let _glob1_ref = glob1.to_ref();
    drop(glob1);
    app.update();
    let glob3 = Glob::<&str>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
}

#[modor::test]
fn recreate_glob_with_dropped_ref_after_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::<&str>::from_app(&mut app);
    let _glob2 = Glob::<&str>::from_app(&mut app);
    let glob1_ref = glob1.to_ref();
    drop(glob1);
    drop(glob1_ref);
    app.update();
    let glob3 = Glob::<&str>::from_app(&mut app);
    assert_eq!(glob3.index(), 2);
    app.update();
    assert_eq!(Glob::<&str>::from_app(&mut app).index(), 0);
    assert_eq!(Glob::<&str>::from_app(&mut app).index(), 3);
}

#[modor::test]
fn access_all_globals() {
    let mut app = App::new::<Root>(Level::Info);
    let glob1 = Glob::from_app(&mut app);
    let glob2 = Glob::from_app(&mut app);
    *glob1.get_mut(&mut app) = "a";
    *glob2.get_mut(&mut app) = "b";
    let globals = app.get_mut::<Globals<&str>>();
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
    let glob1 = Glob::from_app(&mut app);
    let glob2 = Glob::from_app(&mut app);
    *glob1.get_mut(&mut app) = "a";
    *glob2.get_mut(&mut app) = "b";
    drop(glob1);
    app.update();
    let globals = app.get_mut::<Globals<&str>>();
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
fn take_glob() {
    let mut app = App::new::<Root>(Level::Info);
    let mut glob1 = Glob::from_app(&mut app);
    let glob2 = Glob::from_app(&mut app);
    *glob1.get_mut(&mut app) = "a";
    *glob2.get_mut(&mut app) = 42;
    let result = glob1.take(&mut app, |glob1, app| {
        assert_eq!(glob1, &"a");
        assert_eq!(glob2.get_mut(app), &42);
        42
    });
    assert_eq!(result, 42);
}

#[modor::test]
fn access_glob() {}

#[derive(Default, RootNode, Node)]
struct Root;
