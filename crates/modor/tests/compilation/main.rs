#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used)]

#[macro_use]
extern crate modor;

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use modor::{Query, Single, SingleMut, World};
use std::fs;
use std::path::{Path, PathBuf};

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
    #[allow(clippy::trivially_copy_pass_by_ref)]
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
    #[allow(clippy::trivially_copy_pass_by_ref)]
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
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn all_entity_param_types(_: &mut u32, _: Option<&mut i64>, _: &i16, _: Option<&i16>) {}

    #[run]
    fn entity_params_with_query(_: &mut u32, _: Query<'_, (&mut i64,)>) {}

    #[run]
    fn entity_params_with_world(_: &mut u32, _: World<'_>) {}
}

#[test]
fn check_compilation_failures() {
    let root_path = env!("CARGO_MANIFEST_DIR");
    let target_debug_path = [root_path, "..", "..", "target", "debug"]
        .iter()
        .collect::<PathBuf>();
    let target_deps_path = [root_path, "..", "..", "target", "debug", "deps"]
        .iter()
        .collect::<PathBuf>();
    let config = Config {
        mode: Mode::CompileFail,
        src_base: [root_path, "tests", "compilation", "compile_fail"]
            .iter()
            .collect(),
        target_rustcflags: Some(format!(
            "-L {} -L {}",
            target_debug_path.display(),
            target_deps_path.display()
        )),
        ..Config::default()
    };
    move_rmeta_files(&target_deps_path);
    let result = std::panic::catch_unwind(|| compiletest_rs::run_tests(&config));
    restore_rmeta_files(&target_deps_path);
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn move_rmeta_files(target_deps_path: &Path) {
    let target_dir = target_deps_path.parent().and_then(Path::parent).unwrap();
    if let Ok(mut entries) = fs::read_dir(target_deps_path) {
        while let Some(Ok(entry)) = entries.next() {
            let filename = entry.file_name();
            let filename = filename.to_string_lossy();
            let filename = filename.as_ref();
            if filename.starts_with("libmodor") && filename.ends_with(".rmeta") {
                let _result = fs::rename(entry.path(), target_dir.join(filename));
            }
        }
    }
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn restore_rmeta_files(target_deps_path: &Path) {
    let target_dir = target_deps_path.parent().and_then(Path::parent).unwrap();
    if let Ok(mut entries) = fs::read_dir(target_dir) {
        while let Some(Ok(entry)) = entries.next() {
            let filename = entry.file_name();
            let filename = filename.to_string_lossy();
            let filename = filename.as_ref();
            if filename.starts_with("libmodor") && filename.ends_with(".rmeta") {
                let _result = fs::rename(entry.path(), target_deps_path.join(filename));
            }
        }
    }
}
