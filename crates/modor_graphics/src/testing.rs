use crate::Capture;
use image::ColorType;
use modor::testing::TestApp;
use std::path::Path;

pub fn assert_capture<P>(app: &TestApp, capture_path: P)
where
    P: AsRef<Path>,
{
    app.assert_singleton::<Capture>().has(|c: &Capture| {
        let buffer = c
            .buffer()
            .expect("capture not yet done (at least one update required)");
        let size = c.size();
        if !capture_path.as_ref().exists() {
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
        assert_eq!(size.width, image.width());
        assert_eq!(size.height, image.height());
        assert!(
            !buffer
                .iter()
                .zip(image.as_bytes())
                .any(|(a, b)| a.abs_diff(*b) > 1),
            "captures are different"
        );
    });
}
