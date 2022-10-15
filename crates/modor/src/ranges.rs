use std::fmt::Debug;
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// A trait implemented for any `usize` range (`usize` type included).
pub trait UsizeRange: Debug {
    /// Returns whether the `value` is included in the range.
    fn contains_value(&self, value: usize) -> bool;
}

impl UsizeRange for usize {
    fn contains_value(&self, value: usize) -> bool {
        self == &value
    }
}

impl UsizeRange for RangeFull {
    fn contains_value(&self, _value: usize) -> bool {
        true
    }
}

impl UsizeRange for Range<usize> {
    fn contains_value(&self, value: usize) -> bool {
        self.contains(&value)
    }
}

impl UsizeRange for RangeInclusive<usize> {
    fn contains_value(&self, value: usize) -> bool {
        self.contains(&value)
    }
}

impl UsizeRange for RangeFrom<usize> {
    fn contains_value(&self, value: usize) -> bool {
        self.contains(&value)
    }
}

impl UsizeRange for RangeTo<usize> {
    fn contains_value(&self, value: usize) -> bool {
        self.contains(&value)
    }
}

impl UsizeRange for RangeToInclusive<usize> {
    fn contains_value(&self, value: usize) -> bool {
        self.contains(&value)
    }
}
