use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct MissingComponentError;

impl Error for MissingComponentError {}

impl Display for MissingComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "internal error: component does not exist")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(in super::super) struct ExistingComponentError;

impl Error for ExistingComponentError {}

impl Display for ExistingComponentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "internal error: entity has already a component of the same type"
        )
    }
}
