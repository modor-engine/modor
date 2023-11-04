#![allow(missing_docs)]

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    modor_examples::keyboard::main();
}
