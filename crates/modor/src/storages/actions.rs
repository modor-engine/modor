use crate::{App, DynId};
use derivative::Derivative;
use std::cell::RefCell;
use std::mem;

#[derive(Default, Derivative)]
#[derivative(Debug)]
pub(crate) struct ActionStorage {
    #[derivative(Debug = "ignore")]
    actions: RefCell<Vec<Action>>,
}

impl ActionStorage {
    pub(crate) fn push(&self, action: Action) {
        self.actions.borrow_mut().push(action);
    }

    pub(crate) fn take(&mut self) -> Vec<Action> {
        mem::take(self.actions.get_mut())
    }
}

pub(crate) enum Action {
    ObjectDeletion(DynId),
    Other(OtherAction),
}

pub(crate) type OtherAction = Box<dyn FnOnce(&mut App) -> crate::Result<()>>;
