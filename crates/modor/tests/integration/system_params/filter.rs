use crate::system_params::Value;
use modor::{Filter, With};

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel_with_const_param() {
    modor_internal::retry!(
        60,
        assert!(are_systems_run_in_parallel!((), Filter<With<Value>>))
    );
}
