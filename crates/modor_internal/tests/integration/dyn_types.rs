use modor_internal::dyn_types::DynType;
use static_assertions::assert_impl_all;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

assert_impl_all!(
    DynType: Any,
    Clone,
    Hash,
    PartialEq,
    Eq,
    Debug,
    Sync,
    Send,
    UnwindSafe,
    RefUnwindSafe
);

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn compare_dyn_types() {
    let value = DynType::new(0_usize);
    let same_value = DynType::new(0_usize);
    let different_value = DynType::new(1_usize);
    let different_type = DynType::new(String::from("0"));
    assert_eq!(value, same_value);
    assert_ne!(value, different_value);
    assert_ne!(value, different_type);
}
