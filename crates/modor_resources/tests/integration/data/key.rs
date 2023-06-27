use modor_resources::{IndexResKey, ResKey, ResKeyId, Resource, ResourceKey, ResourceState};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[modor_test]
fn create_labeled_keys() {
    let key = ResKey::<TestResource>::new("id1");
    let same_key = ResKey::<TestResource>::new("id1");
    let different_key = ResKey::<TestResource>::new("id2");
    assert_eq!(key, same_key);
    assert_ne!(key, different_key);
    assert_eq!(hash(key), hash(same_key));
    assert_ne!(hash(key), hash(different_key));
    assert_eq!(key, key.clone());
    assert_eq!(key.id(), ResKeyId::Label("id1"));
    assert_eq!(key.label(), "id1");
}

#[modor_test]
fn create_unique_keys() {
    let key = ResKey::<TestResource>::unique("label1");
    let same_label_key = ResKey::<TestResource>::unique("label1");
    let different_label_key = ResKey::<TestResource>::unique("label2");
    let labeled_key = ResKey::<TestResource>::new("label1");
    assert_ne!(key, same_label_key);
    assert_ne!(key, different_label_key);
    assert_ne!(hash(key), hash(same_label_key));
    assert_ne!(hash(key), hash(different_label_key));
    assert_ne!(hash(key), hash(labeled_key));
    assert_ne!(key, labeled_key);
    assert_ne!(key, labeled_key.clone());
    assert_eq!(key.id(), ResKeyId::Index(0));
    assert_eq!(same_label_key.id(), ResKeyId::Index(1));
    assert_eq!(different_label_key.id(), ResKeyId::Index(2));
    assert_eq!(key.label(), "label1#0");
    assert_eq!(same_label_key.label(), "label1#1");
    assert_eq!(different_label_key.label(), "label2#2");
}

#[modor_test]
fn create_indexed_keys() {
    let indexer = IndexResKey::<TestResource>::new("id1");
    let same_indexer = IndexResKey::<TestResource>::new("id1");
    let different_indexer = IndexResKey::<TestResource>::new("id2");
    assert_eq!(indexer.get(0), indexer.get(0));
    assert_eq!(indexer.get(0), same_indexer.get(0));
    assert_ne!(indexer.get(0), indexer.get(1));
    assert_ne!(indexer.get(0), different_indexer.get(0));
    assert_eq!(hash(indexer.get(0)), hash(same_indexer.get(0)));
    assert_ne!(hash(indexer.get(0)), hash(different_indexer.get(0)));
    assert_eq!(indexer.get(0).id(), ResKeyId::LabeledIndex("id1", 0));
    assert_eq!(indexer.get(1).id(), ResKeyId::LabeledIndex("id1", 1));
    assert_eq!(indexer.get(0).label(), "id1.0");
    assert_eq!(indexer.get(1).label(), "id1.1");
}

fn hash(key: ResKey<TestResource>) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

struct TestResource;

impl Resource for TestResource {
    fn key(&self) -> &ResourceKey {
        unimplemented!()
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::NotLoaded
    }
}

#[modor_test]
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

#[modor_test]
fn retrieve_debug_key_value() {
    let key1 = ResourceKey::new(1_u32);
    let key2 = ResourceKey::new(2_i8);
    let key3 = ResourceKey::new(2_i8);
    assert_ne!(format!("{key1:?}"), format!("{key2:?}"));
    assert_eq!(format!("{key2:?}"), format!("{key3:?}"));
}
