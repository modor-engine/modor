#![allow(clippy::trivially_copy_pass_by_ref)]

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use modor::{system, EntityMainComponent, Query, Single, SingleMut, Singleton, World};
use std::path::PathBuf;

struct SingletonEntity1;

impl EntityMainComponent for SingletonEntity1 {
    type Type = Singleton;
}

struct SingletonEntity2;

impl EntityMainComponent for SingletonEntity2 {
    type Type = Singleton;
}

fn no_param() {}

fn mandatory_component(_: &u32) {}

fn mandatory_component_in_tuple(_: Option<&u32>, _: (&mut i64,)) {}

fn mandatory_component_in_sub_tuple(_: Option<&u32>, _: (World<'_>, (&mut i64,))) {}

fn mandatory_component_in_query(_: Query<'_, &mut i64>) {}

fn mandatory_component_in_tuple_in_query(_: Query<'_, (&mut i64,)>) {}

fn mandatory_component_in_sub_tuple_in_query(_: Query<'_, (&u32, (&mut i64,))>) {}

fn same_const_component(_: &u32, _: &u32) {}

fn different_mut_components(_: &mut u32, _: &mut i64) {}

fn same_const_singleton(
    _: Single<'_, SingletonEntity1>,
    _: Single<'_, SingletonEntity1>,
    _: &SingletonEntity1,
) {
}

fn different_mut_singletons(
    _: SingleMut<'_, SingletonEntity1>,
    _: SingleMut<'_, SingletonEntity2>,
) {
}

fn same_const_singleton_option(
    _: Option<Single<'_, SingletonEntity1>>,
    _: Option<Single<'_, SingletonEntity1>>,
    _: &SingletonEntity1,
) {
}

fn different_mut_singleton_options(
    _: Option<SingleMut<'_, SingletonEntity1>>,
    _: Option<SingleMut<'_, SingletonEntity2>>,
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
    let root_path = env!("CARGO_MANIFEST_DIR");
    let lib_path_1 = [root_path, "..", "..", "target", "debug"]
        .iter()
        .collect::<PathBuf>();
    let lib_path_2 = [root_path, "..", "..", "target", "debug", "deps"]
        .iter()
        .collect::<PathBuf>();
    let config = Config {
        mode: Mode::CompileFail,
        src_base: [root_path, "tests", "system_static_check"].iter().collect(),
        target_rustcflags: Some(format!(
            "-L {} -L {}",
            lib_path_1.display(),
            lib_path_2.display()
        )),
        ..Config::default()
    };
    config.clean_rmeta();
    compiletest_rs::run_tests(&config);
}
