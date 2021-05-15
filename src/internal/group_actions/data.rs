use crate::internal::main::MainFacade;
use crate::GroupBuilder;

pub(in super::super) type BuildGroupFn = Box<dyn FnOnce(&mut GroupBuilder<'_>) + Sync + Send>;
pub(in super::super) type CreateEntityFn = Box<dyn FnOnce(&mut MainFacade) + Sync + Send>;

pub(super) enum GroupBuilderState {
    Some(BuildGroupFn),
    Removed,
    None,
}
