#![allow(missing_docs)]

use fs_extra::dir::CopyOptions;
use pico_args::Arguments;
use std::path::Path;

fn main() {
    let mut args = Arguments::from_env();
    let _release = args.contains("--release");
    let _example = args.contains("--example");
    let _build_only = args.contains("--build-only");
    let _features: Result<Option<String>, _> = args.opt_value_from_str("--features");
    let _host: Result<Option<String>, _> = args.opt_value_from_str("--host");
    let _port: Result<Option<String>, _> = args.opt_value_from_str("--port");
    if let Some(name) = args.finish().first() {
        let source_folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../examples")
            .join("assets");
        if source_folder.exists() {
            let destination_folder = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../target/wasm-examples")
                .join(name);
            fs_extra::dir::create_all(&destination_folder, true)
                .expect("cannot create asset folder");
            fs_extra::copy_items(&[source_folder], &destination_folder, &CopyOptions::new())
                .expect("cannot copy asset folder");
        }
    }
    cargo_run_wasm::run_wasm_cli_with_css("");
}
