use modor_resources::{IndexResKey, ResKey, Resource, ResourceState};
use std::collections::hash_map::DefaultHasher;
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
    assert_eq!(indexer.get(0).label(), "id1.0");
    assert_eq!(indexer.get(1).label(), "id1.1");
}

fn hash(key: ResKey<TestResource>) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

struct TestResource(ResKey<Self>);

impl Resource for TestResource {
    fn key(&self) -> ResKey<Self> {
        self.0
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::NotLoaded
    }
}
