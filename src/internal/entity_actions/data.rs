use crate::internal::main::MainFacade;

pub(in super::super) type AddComponentFn = Box<dyn FnOnce(&mut MainFacade) + Sync + Send>;
