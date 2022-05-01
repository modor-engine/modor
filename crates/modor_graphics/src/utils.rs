pub(crate) fn nearest_multiple(value: u64, multiple: u64) -> u64 {
    let align_mask = multiple - 1;
    (value + align_mask) & !align_mask
}

pub(crate) fn normalize(
    value: f32,
    min: f32,
    max: f32,
    normalized_min: f32,
    normalized_max: f32,
) -> f32 {
    if max > min {
        ((value - min) / (max - min)).mul_add(normalized_max - normalized_min, normalized_min)
    } else {
        normalized_min
    }
}

#[cfg(test)]
mod utils_tests {
    use approx::assert_abs_diff_eq;

    #[test]
    fn calculate_nearest_multiple() {
        assert_eq!(super::nearest_multiple(0, 4), 0);
        assert_eq!(super::nearest_multiple(1, 4), 4);
        assert_eq!(super::nearest_multiple(4, 4), 4);
        assert_eq!(super::nearest_multiple(5, 4), 8);
    }

    #[test]
    fn normalize() {
        assert_abs_diff_eq!(super::normalize(-1., -1., 1., 1., 2.), 1.);
        assert_abs_diff_eq!(super::normalize(0., -1., 1., 1., 2.), 1.5);
        assert_abs_diff_eq!(super::normalize(1., -1., 1., 1., 2.), 2.);
        assert_abs_diff_eq!(super::normalize(0., 0., 0., 1., 2.), 1.);
    }
}
