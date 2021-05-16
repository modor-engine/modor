use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use std::num::NonZeroUsize;

pub(in super::super) struct ActionResult {
    pub(in super::super) deleted_group_idxs: Vec<NonZeroUsize>,
    pub(in super::super) replaced_group_builders: Vec<(NonZeroUsize, BuildGroupFn)>,
    pub(in super::super) entity_builders: Vec<CreateEntityFn>,
    pub(in super::super) deleted_entity_idxs: Vec<usize>,
}
