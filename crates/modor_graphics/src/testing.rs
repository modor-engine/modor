//! Testing utilities.

use crate::Capture;
use image::ColorType;
use modor::{EntityAssertions, With};
use std::panic::RefUnwindSafe;
use std::path::Path;

const COLOR_EPSILON: u8 = 1;

/// Asserts the generated [`Capture`](crate::Capture) is the same as the image saved at
/// `capture_path`.
///
/// If the image does not exist, the generated capture is saved at `capture_path`.
///
/// # Panics
///
/// This will panic if:
/// - the generated capture is different than the image located at `capture_path`
/// - the parent of `capture_path` does not exist or is not a folder
/// - image located at `capture_path` cannot be read
///
/// # Examples
///
/// ```rust
/// # use modor::{App, With};
/// # use modor_graphics::{Capture, GraphicsModule, SurfaceSize};
/// # use modor_graphics::testing::assert_capture;
/// #
/// # fn f() {
/// App::new()
///     .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
///     .assert::<With<Capture>>(1, |e| assert_capture(e, "tests/expected/screen.png"));
/// # }
/// ```
pub fn assert_capture<P>(
    entity: EntityAssertions<'_, With<Capture>>,
    capture_path: P,
) -> EntityAssertions<'_, With<Capture>>
where
    P: AsRef<Path> + RefUnwindSafe,
{
    entity.has(|c: &Capture| {
        let buffer = c
            .buffer()
            .expect("capture not yet done (at least one update required)");
        if !capture_path.as_ref().exists() {
            let size = c.size();
            image::save_buffer(
                capture_path.as_ref(),
                buffer,
                size.width,
                size.height,
                ColorType::Rgba8,
            )
            .expect("cannot save expected capture");
        }
        let image =
            image::open(capture_path.as_ref()).expect("cannot read expected capture from disk");
        assert!(
            !buffer
                .iter()
                .zip(image.as_bytes())
                .any(|(a, b)| a.abs_diff(*b) > COLOR_EPSILON),
            "captures are different"
        );
    })
}
