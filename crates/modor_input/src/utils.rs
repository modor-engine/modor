use modor_math::Vec2;

#[allow(clippy::fn_params_excessive_bools)]
pub(crate) fn normalized_direction(left: bool, right: bool, up: bool, down: bool) -> Vec2 {
    let mut delta = Vec2::xy(0., 0.);
    if left {
        delta.x -= 1.;
    }
    if right {
        delta.x += 1.;
    }
    if up {
        delta.y += 1.;
    }
    if down {
        delta.y -= 1.;
    }
    delta.with_magnitude(1.).unwrap_or(Vec2::ZERO)
}
