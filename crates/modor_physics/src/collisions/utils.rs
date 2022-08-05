pub(crate) fn is_between(value: f32, first: f32, last: f32) -> bool {
    (value >= first && value <= last) || (value >= last && value <= first)
}
