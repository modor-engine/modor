use modor::DynKey;
use std::collections::HashMap;

#[test]
fn use_dynamic_hashmap_keys() {
    let mut map = HashMap::<DynKey, u32>::new();
    let key1 = DynKey::new(12_usize);
    let key2 = DynKey::new("key");
    map.insert(key1.clone(), 0);
    map.insert(key2.clone(), 1);
    assert_eq!(map.get(&key1), Some(&0_u32));
    assert_eq!(map.get(&key2), Some(&1_u32));
    assert_eq!(map.get(&DynKey::new(33_usize)), None);
    assert_eq!(map.get(&DynKey::new("other key")), None);
}
