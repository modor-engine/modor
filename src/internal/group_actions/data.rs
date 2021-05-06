use crate::GroupBuilder;

pub(in super::super) type BuildGroupFn = Box<dyn FnOnce(&mut GroupBuilder<'_>) + Sync + Send>;
