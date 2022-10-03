#![allow(clippy::unwrap_used)]

macro_rules! assert_approx_eq {
    ($left:expr, $right:expr) => {
        approx::assert_abs_diff_eq!($left, $right, epsilon = 0.000_01)
    };
}

#[macro_use]
extern crate modor;

pub mod components;
pub mod entities;
