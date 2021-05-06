#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in super::super) enum LockedSystem {
    Done,
    None,
    Some(SystemLocation),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in super::super) struct SystemLocation {
    pub(in super::super) group_idx: usize,
    pub(in super::super) system_idx: usize,
}

impl SystemLocation {
    pub(in super::super) fn new(group_idx: usize, system_idx: usize) -> Self {
        Self {
            group_idx,
            system_idx,
        }
    }
}

pub(super) enum TypeState {
    Free,
    Read(usize),
    Written,
}
