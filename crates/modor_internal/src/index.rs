use log::error;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

const ERROR: &str = "cannot access index pool";

#[derive(Debug, Default)]
pub struct IndexPool {
    deleted_indexes: Mutex<Vec<usize>>,
    available_indexes: Mutex<Vec<usize>>,
    next_index: AtomicUsize,
}

impl IndexPool {
    pub fn generate(self: &Arc<Self>) -> Index {
        Index {
            index: if let Some(index) = self.available_indexes.lock().expect(ERROR).pop() {
                index
            } else {
                self.next_index.fetch_add(1, Ordering::Relaxed)
            },
            pool: self.clone(),
        }
    }

    pub fn take_deleted_indexes(&self) -> Vec<usize> {
        let indexes = mem::take(&mut *self.deleted_indexes.lock().expect(ERROR));
        self.available_indexes
            .lock()
            .expect(ERROR)
            .extend_from_slice(&indexes);
        indexes
    }
}

#[derive(Debug)]
pub struct Index {
    index: usize,
    pool: Arc<IndexPool>,
}

impl Index {
    pub fn value(&self) -> usize {
        self.index
    }
}

impl Drop for Index {
    fn drop(&mut self) {
        match self.pool.deleted_indexes.lock() {
            Ok(mut indexes) => indexes.push(self.index),
            Err(err) => error!("error: {err}"), // no-coverage (difficult to test poisoning)
        }
    }
}
