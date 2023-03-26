#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    modor_examples::rendering_2d_new::main();
}
