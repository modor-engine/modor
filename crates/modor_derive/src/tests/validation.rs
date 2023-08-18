use crate::tests::generation;
use crate::tests::parsing::TestArgs;
use proc_macro_error::abort;

pub(super) fn check_platform_paths(args: &TestArgs) {
    for platform in args.disabled.iter() {
        if platform.segments.len() > 1 {
            abort!(
                platform,
                "allowed platforms are {:?}",
                generation::supported_platforms()
            );
        }
    }
}
