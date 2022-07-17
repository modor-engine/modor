pub(crate) fn is_between(value: f32, first: f32, last: f32) -> bool {
    (value >= first && value <= last) || (value >= last && value <= first)
}

pub(crate) fn is_almost_eq(value1: f32, value2: f32) -> bool {
    is_between(value1, value2 - f32::EPSILON, value2 + f32::EPSILON)
}
