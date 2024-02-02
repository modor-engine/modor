#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use std::fs;
use std::path::{Path, PathBuf};

#[modor::test(disabled(wasm))]
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
