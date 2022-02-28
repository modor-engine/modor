use compiletest_rs::common::Mode;
use compiletest_rs::Config;
use std::path::PathBuf;

#[test]
fn create_invalid_actions() {
    let config = Config {
        mode: Mode::CompileFail,
        src_base: PathBuf::from("action_static_check"),
        target_rustcflags: Some("-L target/debug -L target/debug/deps".to_string()),
        ..Config::default()
    };
    config.clean_rmeta();
    compiletest_rs::run_tests(&config);
}
