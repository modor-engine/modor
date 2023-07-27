use log::Level;
use std::marker::PhantomData;
use std::panic;

pub(crate) fn check_catch_unwind_availability() {
    panic!("`panic::catch_unwind` unsupported on this platform");
}

pub(crate) fn init_logging(level: Level) {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(level).expect("cannot initialize logger");
}

/// A trait implemented for any type implementing [`Sync`], or implemented for any type on Web
/// platform.
pub trait VariableSync {}

impl<T> VariableSync for T {}

/// A trait implemented for any type implementing [`Send`], or implemented for any type on Web
/// platform.
pub trait VariableSend {}

impl<T> VariableSend for T {}

pub(crate) struct Pool(PhantomData<()>);

#[allow(clippy::unused_self)]
impl Pool {
    pub(crate) fn new(_thread_count: u32) -> Option<Self> {
        None
    }

    pub(crate) fn thread_count(&self) -> u32 {
        unreachable!()
    }

    pub(crate) fn scoped<'pool, 'scope, F, R>(&'pool mut self, _f: F) -> R
    where
        F: FnOnce(&Scope<'pool, 'scope>) -> R,
    {
        unreachable!()
    }
}

pub(crate) struct Scope<'pool, 'scope>(PhantomData<(&'pool (), &'scope ())>);

#[allow(clippy::unused_self)]
impl<'pool, 'scope> Scope<'pool, 'scope> {
    pub(crate) fn execute<F>(&self, _f: F)
    where
        F: FnOnce() + 'scope,
    {
        unreachable!()
    }
}
