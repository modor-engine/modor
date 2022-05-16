use modor_internal::{idx_type, ti_vec};
use typed_index_collections::TiVec;

idx_type!(TestIdx);

#[test]
fn create_idx_type() {
    let idx = TestIdx::from(10);
    assert_eq!(idx.0, 10);
    assert_eq!(usize::from(idx), 10);
}

#[test]
fn create_ti_vec() {
    let vec: TiVec<u32, i64> = ti_vec![];
    assert_eq!(vec.len(), 0);
    let vec: TiVec<u32, i64> = ti_vec![1, 2];
    assert_eq!(vec.into_iter().collect::<Vec<_>>(), vec![1, 2]);
    let vec: TiVec<u32, i64> = ti_vec![1; 2];
    assert_eq!(vec.into_iter().collect::<Vec<_>>(), vec![1, 1]);
}

#[test]
fn set_ti_vec_values() {
    let mut vec = TiVec::<usize, usize>::new();
    ti_vec::set_value(&mut vec, 2, 10);
    ti_vec::set_value(&mut vec, 1, 20);
    assert_eq!(vec, ti_vec![0, 20, 10]);
}
