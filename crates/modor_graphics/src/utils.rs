
pub(crate) fn nearest_u64_multiple(value: u64, multiple: u64) -> u64 {
    let align_mask = multiple - 1;
    (value + align_mask) & !align_mask
}

#[cfg(test)]
mod utils_tests {
    #[test]
    fn calculate_nearest_u64_multiple() {
        assert_eq!(super::nearest_u64_multiple(0, 4), 0);
        assert_eq!(super::nearest_u64_multiple(1, 4), 4);
        assert_eq!(super::nearest_u64_multiple(4, 4), 4);
        assert_eq!(super::nearest_u64_multiple(5, 4), 8);
    }
}
