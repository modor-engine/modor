use scoped_threadpool::Scope;

/// A trait implemented for any type implementing [`Sync`], or implemented for any type on Web
/// platform.
pub trait VariableSync: Sync {}

impl<T> VariableSync for T where T: Sync {}

/// A trait implemented for any type implementing [`Send`], or implemented for any type on Web
/// platform.
pub trait VariableSend: Send {}

impl<T> VariableSend for T where T: Send {}

pub(crate) struct Pool {
    inner: scoped_threadpool::Pool,
}

impl Pool {
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn new(thread_count: u32) -> Option<Self> {
        Some(Self {
            inner: scoped_threadpool::Pool::new(thread_count),
        })
    }

    pub(crate) fn thread_count(&self) -> u32 {
        self.inner.thread_count()
    }

    pub(crate) fn scoped<'pool, 'scope, F, R>(&'pool mut self, f: F) -> R
    where
        F: FnOnce(&Scope<'pool, 'scope>) -> R,
    {
        self.inner.scoped(f)
    }
}
