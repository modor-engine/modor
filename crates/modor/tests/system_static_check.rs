#![allow(clippy::trivially_copy_pass_by_ref)]
#![cfg(not(target_arch = "wasm32"))]

#[macro_use]
extern crate modor;

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use modor::{Query, Single, SingleMut, World};
use std::fs;
use std::path::PathBuf;

struct SingletonEntity1;

#[singleton]
impl SingletonEntity1 {}

struct SingletonEntity2;

#[singleton]
impl SingletonEntity2 {}

struct EntityWithValidSystems;

#[entity]
impl EntityWithValidSystems {
    #[run]
    fn no_param() {}

    #[run]
    fn mandatory_component(_: &u32) {}

    #[run]
    fn mandatory_component_in_tuple(_: Option<&u32>, _: (&mut i64,)) {}

    #[run]
    fn mandatory_component_in_sub_tuple(_: Option<&u32>, _: (World<'_>, (&mut i64,))) {}

    #[run]
    fn mandatory_component_in_query(_: Query<'_, &mut i64>) {}

    #[run]
    fn mandatory_component_in_tuple_in_query(_: Query<'_, (&mut i64,)>) {}

    #[run]
    fn mandatory_component_in_sub_tuple_in_query(_: Query<'_, (&u32, (&mut i64,))>) {}

    #[run]
    fn same_const_component(_: &u32, _: &u32) {}

    #[run]
    fn different_mut_components(_: &mut u32, _: &mut i64) {}

    #[run]
    fn same_const_singleton(
        _: Single<'_, SingletonEntity1>,
        _: Single<'_, SingletonEntity1>,
        _: &SingletonEntity1,
    ) {
    }

    #[run]
    fn different_mut_singletons(
        _: SingleMut<'_, SingletonEntity1>,
        _: SingleMut<'_, SingletonEntity2>,
    ) {
    }

    #[run]
    fn same_const_singleton_option(
        _: Option<Single<'_, SingletonEntity1>>,
        _: Option<Single<'_, SingletonEntity1>>,
        _: &SingletonEntity1,
    ) {
    }

    #[run]
    fn different_mut_singleton_options(
        _: Option<SingleMut<'_, SingletonEntity1>>,
        _: Option<SingleMut<'_, SingletonEntity2>>,
    ) {
    }

    #[run]
    fn all_entity_param_types(_: &mut u32, _: Option<&mut i64>, _: &i16, _: Option<&i16>) {}

    #[run]
    fn entity_params_with_query(_: &mut u32, _: Query<'_, (&mut i64,)>) {}

    #[run]
    fn entity_params_with_world(_: &mut u32, _: World<'_>) {}
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
    clean_rmeta(&config);
    compiletest_rs::run_tests(&config);
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn clean_rmeta(config: &Config) {
    if config.target_rustcflags.is_some() {
        for directory in config
            .target_rustcflags
            .as_ref()
            .expect("cannot retrieve rustc flags")
            .split_whitespace()
            .filter(|s| s.ends_with("/deps"))
        {
            if let Ok(mut entries) = fs::read_dir(directory) {
                while let Some(Ok(entry)) = entries.next() {
                    let has_rmeta_extension = entry
                        .file_name()
                        .to_string_lossy()
                        .as_ref()
                        .ends_with(".rmeta");
                    let is_modor = entry
                        .file_name()
                        .to_string_lossy()
                        .as_ref()
                        .starts_with("libmodor");
                    if has_rmeta_extension && is_modor {
                        let _result = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}
