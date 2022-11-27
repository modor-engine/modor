#[allow(clippy::cast_precision_loss)]
pub(crate) fn world_scale(window_size: (u32, u32)) -> (f32, f32) {
    (
        f32::min(window_size.1 as f32 / window_size.0 as f32, 1.),
        f32::min(window_size.0 as f32 / window_size.1 as f32, 1.),
    )
}

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
#[cfg(not(target_arch = "wasm32"))]
mod numbers_tests {
    #[test]
    fn calculate_world_scale() {
        assert_approx_eq!(super::world_scale((800, 600)).0, 0.75);
        assert_approx_eq!(super::world_scale((800, 600)).1, 1.);
        assert_approx_eq!(super::world_scale((600, 800)).0, 1.);
        assert_approx_eq!(super::world_scale((600, 800)).1, 0.75);
        assert_approx_eq!(super::world_scale((800, 800)).0, 1.);
        assert_approx_eq!(super::world_scale((800, 800)).1, 1.);
    }

    #[test]
    fn calculate_nearest_multiple() {
        assert_eq!(super::nearest_multiple(0, 4), 0);
        assert_eq!(super::nearest_multiple(1, 4), 4);
        assert_eq!(super::nearest_multiple(4, 4), 4);
        assert_eq!(super::nearest_multiple(5, 4), 8);
    }

    #[test]
    fn normalize() {
        assert_approx_eq!(super::normalize(-1., -1., 1., 1., 2.), 1.);
        assert_approx_eq!(super::normalize(0., -1., 1., 1., 2.), 1.5);
        assert_approx_eq!(super::normalize(1., -1., 1., 1., 2.), 2.);
        assert_approx_eq!(super::normalize(0., 0., 0., 1., 2.), 1.);
    }
}
