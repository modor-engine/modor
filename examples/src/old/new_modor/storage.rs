use crate::new_modor::Data;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut, Range};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub(crate) struct Storage<T> {
    items: Vec<T>,
}

impl<T> Storage<T>
where
    T: Data,
{
    pub(crate) fn get_mut(&mut self, index: usize) -> &mut T {
        (self.items.len()..=index).for_each(|_| self.items.push(T::default()));
        &mut self.items[index]
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut()
    }

    pub(crate) fn range_iter_mut(&mut self, range: Range<usize>) -> impl Iterator<Item = &mut T> {
        (self.items.len()..=range.end).for_each(|_| self.items.push(T::default()));
        self.items[range].iter_mut()
    }
}

// TODO: simplify ?
#[derive(Copy, Clone, Debug, Eq)]
pub struct Scope {
    id: ScopeId,
    hash: u64,
}

impl PartialEq for Scope {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for Scope {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash);
    }
}

impl Scope {
    pub const fn new(id: &'static str) -> Self {
        Self {
            id: ScopeId::Str(id),
            hash: Self::calculate_hash(id),
        }
    }

    pub fn unique() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self {
            id: ScopeId::Usize(id),
            hash: id as u64,
        }
    }

    pub const fn index(self, index: usize) -> Index {
        Index { scope: self, index }
    }

    const fn calculate_hash(id: &'static str) -> u64 {
        let bytes = id.as_bytes();
        let mut hash = if bytes.is_empty() { 0 } else { bytes[0] as u64 };
        if bytes.len() > 1 {
            hash |= (bytes[1] as u64) << 8;
        }
        if bytes.len() > 2 {
            hash |= (bytes[2] as u64) << 16;
        }
        if bytes.len() > 3 {
            hash |= (bytes[3] as u64) << 24;
        }
        if bytes.len() > 4 {
            hash |= (bytes[bytes.len() - 1] as u64) << 32;
        }
        if bytes.len() > 5 {
            hash |= (bytes[bytes.len() - 2] as u64) << 40;
        }
        if bytes.len() > 6 {
            hash |= (bytes[bytes.len() - 3] as u64) << 48;
        }
        if bytes.len() > 7 {
            hash |= (bytes[bytes.len() - 4] as u64) << 56;
        }
        hash
    }
}

#[derive(Copy, Clone, Debug, Eq)]
enum ScopeId {
    Str(&'static str),
    Usize(usize),
}

impl PartialEq for ScopeId {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Usize(index1), Self::Usize(index2)) => index1 == index2,
            (Self::Str(id1), Self::Str(id2)) => id1.as_ptr() == id2.as_ptr() || id1 == id2,
            (Self::Usize(_), Self::Str(_)) | (Self::Str(_), Self::Usize(_)) => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Index {
    pub(crate) scope: Scope,
    pub(crate) index: usize,
}

// TODO: use get() instead ?
pub enum State<T> {
    New(T),
    Existing(T),
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let (Self::New(value) | Self::Existing(value)) = self;
        value
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let (Self::New(value) | Self::Existing(value)) = self;
        value
    }
}

impl<T> State<T> {
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }
}
