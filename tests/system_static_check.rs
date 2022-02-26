#![allow(clippy::trivially_copy_pass_by_ref)]

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use modor::{system, Query, Single, SingleMut, World};
use std::path::PathBuf;

fn no_param() {}

fn mandatory_component(_: &u32) {}

fn mandatory_component_in_tuple(_: Option<&u32>, _: (&mut i64,)) {}

fn mandatory_component_in_sub_tuple(_: Option<&u32>, _: (World<'_>, (&mut i64,))) {}

fn mandatory_component_in_query(_: Query<'_, &mut i64>) {}

fn mandatory_component_in_tuple_in_query(_: Query<'_, (&mut i64,)>) {}

fn mandatory_component_in_sub_tuple_in_query(_: Query<'_, (&u32, (&mut i64,))>) {}

fn same_const_component(_: &u32, _: &u32) {}

fn different_mut_components(_: &mut u32, _: &mut i64) {}

fn same_const_singleton(_: Single<'_, u32>, _: Single<'_, u32>, _: &u32) {}

fn different_mut_singletons(_: SingleMut<'_, u32>, _: SingleMut<'_, i64>) {}

fn same_const_singleton_option(_: Option<Single<'_, u32>>, _: Option<Single<'_, u32>>, _: &u32) {}

fn different_mut_singleton_options(_: Option<SingleMut<'_, u32>>, _: Option<SingleMut<'_, i64>>) {}

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
    system!(same_const_singleton);
    system!(different_mut_singletons);
    system!(same_const_singleton_option);
    system!(different_mut_singleton_options);
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
