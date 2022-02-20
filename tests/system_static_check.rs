#![allow(clippy::trivially_copy_pass_by_ref)]

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use modor::{system, Glob, GlobMut, Global, Query, World};
use std::path::PathBuf;

struct TestGlobal1;

impl Global for TestGlobal1 {}

struct TestGlobal2;

impl Global for TestGlobal2 {}

fn no_param() {}

fn mandatory_component(_: &u32) {}

fn mandatory_component_in_tuple(_: Option<&u32>, _: (&mut i64,)) {}

fn mandatory_component_in_sub_tuple(_: Option<&u32>, _: (World<'_>, (&mut i64,))) {}

fn mandatory_component_in_query(_: Query<'_, &mut i64>) {}

fn mandatory_component_in_tuple_in_query(_: Query<'_, (&mut i64,)>) {}

fn mandatory_component_in_sub_tuple_in_query(_: Query<'_, (&u32, (&mut i64,))>) {}

fn same_const_component(_: &u32, _: &u32) {}

fn different_mut_components(_: &mut u32, _: &mut i64) {}

fn same_const_global(_: Glob<'_, TestGlobal1>, _: Glob<'_, TestGlobal1>) {}

fn different_mut_globals(_: GlobMut<'_, TestGlobal1>, _: GlobMut<'_, TestGlobal2>) {}

fn same_const_global_option(_: Option<Glob<'_, TestGlobal1>>, _: Option<Glob<'_, TestGlobal1>>) {}

fn different_mut_global_options(
    _: Option<GlobMut<'_, TestGlobal1>>,
    _: Option<GlobMut<'_, TestGlobal2>>,
) {
}

fn all_entity_param_types(_: &mut u32, _: Option<&mut i64>, _: &i16, _: Option<&i16>) {}

fn entity_params_with_query(_: &mut u32, _: Query<'_, (&mut i64,)>) {}

fn entity_params_with_world(_: &mut u32, _: World<'_>) {}

#[test]
fn create_valid_systems() {
    system!(no_param);
    system!(mandatory_component);
    system!(mandatory_component_in_tuple);
    system!(mandatory_component_in_sub_tuple);
    system!(mandatory_component_in_query);
    system!(mandatory_component_in_tuple_in_query);
    system!(mandatory_component_in_sub_tuple_in_query);
    system!(same_const_component);
    system!(different_mut_components);
    system!(same_const_global);
    system!(different_mut_globals);
    system!(same_const_global_option);
    system!(different_mut_global_options);
    system!(all_entity_param_types);
    system!(entity_params_with_query);
    system!(entity_params_with_world);
}

#[test]
fn create_invalid_systems() {
    let config = Config {
        mode: Mode::CompileFail,
        src_base: PathBuf::from("tests/system_static_check"),
        target_rustcflags: Some("-L target/debug -L target/debug/deps".to_string()),
        ..Config::default()
    };
    config.clean_rmeta();
    compiletest_rs::run_tests(&config);
}
