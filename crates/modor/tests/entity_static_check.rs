#![cfg(not(target_arch = "wasm32"))]

use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use std::path::PathBuf;
use std::fs;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_invalid_entity() {
    let root_path = env!("CARGO_MANIFEST_DIR");
    let lib_path_1 = [root_path, "..", "..", "target", "debug"]
        .iter()
        .collect::<PathBuf>();
    let lib_path_2 = [root_path, "..", "..", "target", "debug", "deps"]
        .iter()
        .collect::<PathBuf>();
    let config = Config {
        mode: Mode::CompileFail,
        src_base: [root_path, "tests", "entity_static_check"].iter().collect(),
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

pub fn clean_rmeta(config: &Config) {
    if config.target_rustcflags.is_some() {
        for directory in config
            .target_rustcflags
            .as_ref()
            .unwrap()
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
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }
}
