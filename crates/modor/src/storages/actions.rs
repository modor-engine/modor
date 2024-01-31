use crate::{App, DynId};
use derivative::Derivative;
use std::mem;

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub(crate) struct ActionStorage {
    #[derivative(Debug = "ignore")]
    actions: Vec<Action>,
}

impl ActionStorage {
    pub(crate) fn push(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub(crate) fn take(&mut self) -> Vec<Action> {
        mem::take(&mut self.actions)
    }
}

pub(crate) enum Action {
    ObjectDeletion(DynId),
    Other(OtherAction),
}

pub(crate) type OtherAction = Box<dyn FnOnce(&mut App) -> crate::Result<()>>;
