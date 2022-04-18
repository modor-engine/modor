use crate::Capture;
use image::io::Reader;
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
        let image = Reader::open(capture_path.as_ref())
            .expect("cannot read expected capture from disk")
            .decode()
            .expect("cannot decode expected capture");
        assert!(
            !buffer
                .iter()
                .zip(image.as_bytes())
                .any(|(a, b)| a.abs_diff(*b) > 1),
            "captures are different"
        );
    });
}
