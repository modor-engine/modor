use crate::system_param::parsing::SystemParamStruct;
use darling::ast::GenericParamExt;
use proc_macro_error::abort;

pub(super) fn check_lifetime_uniqueness(parsed: &SystemParamStruct) {
    let lifetime_count = parsed
        .input
        .generics
        .params
        .iter()
        .filter(|p| p.as_lifetime_def().is_some())
        .count();
    if lifetime_count > 1 {
        abort!(
            parsed.input.generics,
            "custom system param with more than one generic lifetime",
        );
    }
}
