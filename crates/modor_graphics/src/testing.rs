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
        if !capture_path.as_ref().exists() {
            let size = c.size();
            image::save_buffer(
                capture_path.as_ref(),
                &c.buffer(),
                size.width,
                size.height,
                ColorType::Rgba8,
            )
            .unwrap();
        }
        let image = Reader::open(capture_path.as_ref())
            .unwrap()
            .decode()
            .unwrap();
        assert_eq!(c.buffer(), image.as_bytes());
    });
}
