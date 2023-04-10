use modor_resources::ResourceKey;
use std::collections::HashSet;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_key_in_set() {
    let type1_key1 = ResourceKey::new(11_u32);
    let type1_key2 = ResourceKey::new(12_u32);
    let type2_key1 = ResourceKey::new(21_i8);
    let type2_same_key1 = ResourceKey::new(21_i8);
    let missing_key = ResourceKey::new(3_i8);
    let mut set = HashSet::new();
    set.insert(type1_key1.clone());
    set.insert(type1_key2.clone());
    set.insert(type2_key1.clone());
    set.insert(type2_same_key1.clone());
    assert_eq!(set.len(), 3);
    assert!(set.contains(&type1_key1));
    assert!(set.contains(&type1_key2));
    assert!(set.contains(&type2_key1));
    assert!(set.contains(&type2_same_key1));
    assert!(!set.contains(&missing_key));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn retrieve_debug_key_value() {
    let key1 = ResourceKey::new(1_u32);
    let key2 = ResourceKey::new(2_i8);
    let key3 = ResourceKey::new(2_i8);
    assert_ne!(format!("{key1:?}"), format!("{key2:?}"));
    assert_eq!(format!("{key2:?}"), format!("{key3:?}"));
}
