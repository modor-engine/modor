use modor_graphics_new2::Color;
use modor_internal::assert_approx_eq;

#[modor_test]
fn construct_color_with_red() {
    let color = Color::rgba(1., 0.5, 0.25, 0.15).with_red(0.4);
    assert_approx_eq!(color.r, 0.4);
    assert_approx_eq!(color.g, 0.5);
    assert_approx_eq!(color.b, 0.25);
    assert_approx_eq!(color.a, 0.15);
}

#[modor_test]
fn construct_color_with_green() {
    let color = Color::rgba(1., 0.5, 0.25, 0.15).with_green(0.4);
    assert_approx_eq!(color.r, 1.);
    assert_approx_eq!(color.g, 0.4);
    assert_approx_eq!(color.b, 0.25);
    assert_approx_eq!(color.a, 0.15);
}

#[modor_test]
fn construct_color_with_blue() {
    let color = Color::rgba(1., 0.5, 0.25, 0.15).with_blue(0.4);
    assert_approx_eq!(color.r, 1.);
    assert_approx_eq!(color.g, 0.5);
    assert_approx_eq!(color.b, 0.4);
    assert_approx_eq!(color.a, 0.15);
}

#[modor_test]
fn construct_color_with_alpha() {
    let color = Color::rgba(1., 0.5, 0.25, 0.15).with_alpha(0.4);
    assert_approx_eq!(color.r, 1.);
    assert_approx_eq!(color.g, 0.5);
    assert_approx_eq!(color.b, 0.25);
    assert_approx_eq!(color.a, 0.4);
}
