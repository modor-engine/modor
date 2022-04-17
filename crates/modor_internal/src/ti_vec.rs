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

pub fn set_value<K, V>(vec: &mut TiVec<K, V>, idx: K, value: V)
where
    usize: From<K>,
    K: From<usize>,
    V: Default,
{
    let idx = usize::from(idx);
    (vec.len()..=idx).for_each(|_| vec.push(V::default()));
    vec[K::from(idx)] = value;
}

#[cfg(test)]
mod tests {
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
        super::set_value(&mut vec, 2, 10);
        super::set_value(&mut vec, 1, 20);
        assert_eq!(vec, ti_vec![0, 20, 10]);
    }
}
