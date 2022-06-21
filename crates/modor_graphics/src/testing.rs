//! Testing utilities.

use crate::Capture;
use image::ColorType;
use modor::testing::TestApp;
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
/// - [`Capture`](crate::Capture) has not yet been updated
pub fn assert_capture<P>(app: &TestApp, capture_path: P)
where
    P: AsRef<Path>,
{
    app.assert_singleton::<Capture>().has(|c: &Capture| {
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
    });
}
