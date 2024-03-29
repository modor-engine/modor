use typed_index_collections::TiVec;

#[macro_export]
macro_rules! idx_type {
    ($visibility:vis $name:ident) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        $visibility struct $name($visibility usize);

        impl From<usize> for $name {
            #[inline]
            fn from(idx: usize) -> Self {
                Self(idx)
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(idx: $name) -> Self {
                idx.0
            }
        }
    };
}

#[macro_export]
macro_rules! ti_vec {
    () => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![])
    );
    ($elem:expr; $n:expr) => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![$elem; $n])
    );
    ($($x:expr),+ $(,)?) => (
        ::typed_index_collections::TiVec::<_, _>::from(vec![$($x),+])
    );
}

pub trait TiVecSafeOperations<K, V>
where
    usize: From<K>,
    K: Copy,
    V: Default,
{
    fn get_mut_or_create(&mut self, idx: K) -> &mut V;
}

impl<K, V> TiVecSafeOperations<K, V> for TiVec<K, V>
where
    usize: From<K>,
    K: Copy,
    V: Default,
{
    fn get_mut_or_create(&mut self, idx: K) -> &mut V {
        (self.len()..=idx.into()).for_each(|_| self.push(V::default()));
        &mut self[idx]
    }
}
