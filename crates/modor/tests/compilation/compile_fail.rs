#![cfg(not(target_arch = "wasm32"))]

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use modor::{Filter, Query, With, World};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Component, NoSystem)]
struct C1;

#[derive(Component, NoSystem)]
struct C2;

#[derive(Component, NoSystem)]
struct C3;

#[derive(SingletonComponent, NoSystem)]
struct Singleton1;

#[derive(SingletonComponent, NoSystem)]
struct Singleton2;

#[derive(Component)]
struct EntityWithValidSystems;

#[systems]
impl EntityWithValidSystems {
    #[run]
    fn no_param() {}

    #[run]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn mandatory_component(_: &C2) {}

    #[run]
    fn mandatory_component_in_tuple(_: Option<&C2>, _: (&mut C1,)) {}

    #[run]
    fn mandatory_component_in_sub_tuple(_: Option<&C2>, _: (World<'_>, (&mut C1,))) {}

    #[run]
    fn mandatory_component_in_query(_: Query<'_, &mut C1>) {}

    #[run]
    fn mandatory_component_in_tuple_in_query(_: Query<'_, (&mut C1,)>) {}

    #[run]
    fn mandatory_component_in_sub_tuple_in_query(_: Query<'_, (&C2, (&mut C1,))>) {}

    #[run]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn same_const_component(_: &C2, _: &C2) {}

    #[run]
    fn different_mut_components(_: &mut C2, _: &mut C1) {}

    #[run]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn all_entity_param_types(_: &mut C2, _: Option<&mut C1>, _: &C3, _: Option<&C3>) {}

    #[run]
    fn entity_params_with_query(_: &mut C2, _: Query<'_, (&mut C1,)>) {}

    #[run]
    fn entity_params_with_world(_: &mut C2, _: World<'_>) {}

    #[run]
    fn filters_with_same_component(_: Filter<With<C1>>, _: Filter<With<C1>>) {}
}

#[modor_test(disabled(wasm))]
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
