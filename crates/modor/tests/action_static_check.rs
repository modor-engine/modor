use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use std::path::PathBuf;

#[test]
#[cfg_attr(feature = "test-coverage", ignore)]
fn create_invalid_actions() {
    let root_path = env!("CARGO_MANIFEST_DIR");
    let lib_path_1 = [root_path, "..", "..", "target", "debug"]
        .iter()
        .collect::<PathBuf>();
    let lib_path_2 = [root_path, "..", "..", "target", "debug", "deps"]
        .iter()
        .collect::<PathBuf>();
    let config = Config {
        mode: Mode::CompileFail,
        src_base: [root_path, "tests", "action_static_check"].iter().collect(),
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
