use modor_internal::index::IndexPool;
use std::sync::Arc;

#[test]
fn create_index() {
    let pool = Arc::new(IndexPool::default());
    let index1 = pool.generate();
    let index2 = pool.generate();
    assert_eq!(index1.value(), 0);
    assert_eq!(index2.value(), 1);
}

#[test]
fn delete_index() {
    let pool = Arc::new(IndexPool::default());
    assert!(pool.take_deleted_indexes().is_empty());
    let _index1 = pool.generate();
    let index2 = pool.generate();
    drop(index2);
    let index3 = pool.generate();
    assert_eq!(index3.value(), 2);
    assert_eq!(pool.take_deleted_indexes(), [1]);
    let index4 = pool.generate();
    assert_eq!(index4.value(), 1);
}
