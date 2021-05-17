use std::error::Error;
use std::fmt::{Display, Formatter, Result};

// TODO: rename all "nonexisting" by "missing"
#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct MissingComponentError;

impl Error for MissingComponentError {}

impl Display for MissingComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "internal error: component does not exist")
    }
}
